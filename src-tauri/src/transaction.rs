use crate::AppError;
use crate::planner::MigrationPlan;
use crate::writer::{ExpectedFileState, StagedRun};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionOptions {
    pub run_id: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileAction {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackStatus {
    NotRequired,
    RolledBack,
    ExternalChange,
    RollbackFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupEvidence {
    pub path: PathBuf,
    pub sha256: String,
    pub size: u64,
    pub file_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub destination_path: PathBuf,
    pub relative_path: PathBuf,
    pub operation_ids: Vec<String>,
    pub ownership_run_id: String,
    pub action: FileAction,
    pub precondition_sha256: Option<String>,
    pub pre_run_bytes: Option<Vec<u8>>,
    pub intended_sha256: Option<String>,
    pub intended_bytes: Option<Vec<u8>>,
    pub committed: bool,
    pub committed_sha256: Option<String>,
    pub rollback_status: RollbackStatus,
    pub external_change: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionJournal {
    pub version: u32,
    pub run_id: String,
    pub plan_id: String,
    pub created_at: i64,
    pub destination_root: PathBuf,
    pub backup: BackupEvidence,
    pub entries: Vec<JournalEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitOutcome {
    pub committed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackOutcome {
    pub rolled_back: usize,
    pub external_conflicts: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestorePreviewPath {
    pub path: PathBuf,
    pub action: FileAction,
    pub safe_to_restore: bool,
    pub current_sha256: Option<String>,
    pub expected_committed_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestorePreview {
    pub paths: Vec<RestorePreviewPath>,
}

#[derive(Debug)]
pub struct Transaction {
    options: TransactionOptions,
    run_root: PathBuf,
    journal_path: PathBuf,
    journal: TransactionJournal,
}

impl Transaction {
    pub fn preflight(
        plan: &MigrationPlan,
        staged: &StagedRun,
        destination_root: &Path,
        run_root: &Path,
        options: TransactionOptions,
    ) -> Result<Self, AppError> {
        validate_run_id(&options.run_id)?;
        let destination_root = destination_root
            .canonicalize()
            .map_err(|error| AppError::io(destination_root, error))?;
        if !destination_root.is_dir() {
            return Err(AppError::InvalidProfile(format!(
                "destination is not a directory: {}",
                destination_root.display()
            )));
        }
        let staging_root = staged
            .root
            .canonicalize()
            .map_err(|error| AppError::io(&staged.root, error))?;
        prepare_run_root(run_root)?;
        let run_root = run_root
            .canonicalize()
            .map_err(|error| AppError::io(run_root, error))?;
        if run_root.starts_with(&destination_root) || destination_root.starts_with(&run_root) {
            return Err(AppError::UnsafePath(run_root));
        }
        let backup_path = run_root.join("filament-before-run.zip");
        let backup = create_backup(&destination_root, &backup_path)?;
        let mut entries = Vec::new();
        let mut claimed = BTreeSet::new();

        for artifact in &staged.artifacts {
            validate_relative_path(&artifact.relative_path)?;
            if !claimed.insert(fold_path(&artifact.relative_path)) {
                return Err(AppError::Conflict(format!(
                    "duplicate transaction path: {}",
                    artifact.relative_path.display()
                )));
            }
            let staged_path = staging_root.join(&artifact.relative_path);
            let staged_path = staged_path
                .canonicalize()
                .map_err(|error| AppError::io(&staged_path, error))?;
            if !staged_path.starts_with(&staging_root) || !staged_path.is_file() {
                return Err(AppError::UnsafePath(staged_path));
            }
            let intended_bytes =
                std::fs::read(&staged_path).map_err(|error| AppError::io(&staged_path, error))?;
            let intended_sha256 = sha256_bytes(&intended_bytes);
            if intended_sha256 != artifact.sha256 || intended_bytes.len() as u64 != artifact.size {
                return Err(AppError::Conflict(format!(
                    "staged artifact hash changed: {}",
                    artifact.relative_path.display()
                )));
            }
            let destination_path = guarded_destination(&destination_root, &artifact.relative_path)?;
            let pre_run_bytes = read_optional_file(&destination_path)?;
            let precondition_sha256 = pre_run_bytes.as_deref().map(sha256_bytes);
            let precondition_matches = match &artifact.destination_precondition {
                None => true,
                Some(ExpectedFileState::Absent) => precondition_sha256.is_none(),
                Some(ExpectedFileState::Matches(expected)) => {
                    precondition_sha256.as_ref() == Some(expected)
                }
            };
            if !precondition_matches {
                return Err(AppError::Conflict(format!(
                    "destination precondition mismatch: {}",
                    artifact.relative_path.display()
                )));
            }
            entries.push(JournalEntry {
                destination_path,
                relative_path: artifact.relative_path.clone(),
                operation_ids: artifact.operation_ids.clone(),
                ownership_run_id: options.run_id.clone(),
                action: if pre_run_bytes.is_some() {
                    FileAction::Update
                } else {
                    FileAction::Create
                },
                precondition_sha256,
                pre_run_bytes,
                intended_sha256: Some(intended_sha256),
                intended_bytes: Some(intended_bytes),
                committed: false,
                committed_sha256: None,
                rollback_status: RollbackStatus::NotRequired,
                external_change: None,
            });
        }

        for deletion in &staged.deletions {
            validate_relative_path(&deletion.relative_path)?;
            if !claimed.insert(fold_path(&deletion.relative_path)) {
                return Err(AppError::Conflict(format!(
                    "duplicate transaction path: {}",
                    deletion.relative_path.display()
                )));
            }
            let destination_path = guarded_destination(&destination_root, &deletion.relative_path)?;
            let pre_run_bytes = read_optional_file(&destination_path)?.ok_or_else(|| {
                AppError::Conflict(format!(
                    "delete target is absent: {}",
                    deletion.relative_path.display()
                ))
            })?;
            let precondition_sha256 = sha256_bytes(&pre_run_bytes);
            if precondition_sha256 != deletion.expected_sha256 {
                return Err(AppError::Conflict(format!(
                    "delete precondition mismatch: {}",
                    deletion.relative_path.display()
                )));
            }
            entries.push(JournalEntry {
                destination_path,
                relative_path: deletion.relative_path.clone(),
                operation_ids: deletion.operation_ids.clone(),
                ownership_run_id: options.run_id.clone(),
                action: FileAction::Delete,
                precondition_sha256: Some(precondition_sha256),
                pre_run_bytes: Some(pre_run_bytes),
                intended_sha256: None,
                intended_bytes: None,
                committed: false,
                committed_sha256: None,
                rollback_status: RollbackStatus::NotRequired,
                external_change: None,
            });
        }
        entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        let journal = TransactionJournal {
            version: 1,
            run_id: options.run_id.clone(),
            plan_id: plan.id.clone(),
            created_at: options.created_at,
            destination_root,
            backup,
            entries,
        };
        let journal_path = run_root.join("journal.json");
        persist_json(&journal_path, &journal, &options.run_id)?;
        Ok(Self {
            options,
            run_root,
            journal_path,
            journal,
        })
    }

    pub fn load(run_root: &Path) -> Result<Self, AppError> {
        let run_root = run_root
            .canonicalize()
            .map_err(|error| AppError::io(run_root, error))?;
        let journal_path = run_root.join("journal.json");
        let bytes =
            std::fs::read(&journal_path).map_err(|error| AppError::io(&journal_path, error))?;
        let journal: TransactionJournal =
            serde_json::from_slice(&bytes).map_err(|source| AppError::Json {
                path: journal_path.clone(),
                source,
            })?;
        if journal.version != 1 {
            return Err(AppError::UnsupportedSchema(format!(
                "unsupported transaction journal version: {}",
                journal.version
            )));
        }
        validate_run_id(&journal.run_id)?;
        let destination_root = journal
            .destination_root
            .canonicalize()
            .map_err(|error| AppError::io(&journal.destination_root, error))?;
        if destination_root != journal.destination_root
            || run_root.starts_with(&destination_root)
            || destination_root.starts_with(&run_root)
        {
            return Err(AppError::UnsafePath(journal.destination_root));
        }
        let backup_path = journal
            .backup
            .path
            .canonicalize()
            .map_err(|error| AppError::io(&journal.backup.path, error))?;
        if !backup_path.starts_with(&run_root)
            || sha256_file(&backup_path)? != journal.backup.sha256
        {
            return Err(AppError::Conflict(
                "transaction backup checksum does not match".to_owned(),
            ));
        }
        for entry in &journal.entries {
            validate_relative_path(&entry.relative_path)?;
            if entry.ownership_run_id != journal.run_id
                || guarded_destination(&destination_root, &entry.relative_path)?
                    != entry.destination_path
            {
                return Err(AppError::UnsafePath(entry.destination_path.clone()));
            }
            if entry.pre_run_bytes.as_deref().map(sha256_bytes) != entry.precondition_sha256
                || entry.intended_bytes.as_deref().map(sha256_bytes) != entry.intended_sha256
            {
                return Err(AppError::Conflict(format!(
                    "journal byte checksum mismatch at {}",
                    entry.relative_path.display()
                )));
            }
        }
        let options = TransactionOptions {
            run_id: journal.run_id.clone(),
            created_at: journal.created_at,
        };
        Ok(Self {
            options,
            run_root,
            journal_path,
            journal,
        })
    }

    pub fn commit(&mut self) -> Result<CommitOutcome, AppError> {
        let mut committed = 0;
        for index in 0..self.journal.entries.len() {
            if self.journal.entries[index].committed {
                committed += 1;
                continue;
            }
            if let Err(error) = self.commit_entry(index) {
                let rollback = self.rollback_owned();
                return match rollback {
                    Ok(_) => Err(error),
                    Err(rollback_error) => Err(AppError::Conflict(format!(
                        "{error}; automatic rollback also failed: {rollback_error}"
                    ))),
                };
            }
            committed += 1;
            self.persist()?;
        }
        Ok(CommitOutcome { committed })
    }

    pub fn rollback_owned(&mut self) -> Result<RollbackOutcome, AppError> {
        let mut outcome = RollbackOutcome {
            rolled_back: 0,
            external_conflicts: 0,
            failed: 0,
        };
        for index in (0..self.journal.entries.len()).rev() {
            if !self.journal.entries[index].committed
                || self.journal.entries[index].rollback_status == RollbackStatus::RolledBack
            {
                continue;
            }
            let current = file_state_hash(&self.journal.entries[index].destination_path)?;
            if current != self.journal.entries[index].committed_sha256 {
                let entry = &mut self.journal.entries[index];
                entry.rollback_status = RollbackStatus::ExternalChange;
                entry.external_change = Some(format!(
                    "current state no longer matches committed state at {}",
                    entry.destination_path.display()
                ));
                outcome.external_conflicts += 1;
                self.persist()?;
                continue;
            }
            let destination = self.journal.entries[index].destination_path.clone();
            let prior = self.journal.entries[index].pre_run_bytes.clone();
            let result = match prior {
                Some(bytes) => atomic_write(&destination, &bytes, &self.options.run_id, index),
                None => remove_if_exists(&destination),
            };
            match result {
                Ok(()) => {
                    self.journal.entries[index].rollback_status = RollbackStatus::RolledBack;
                    outcome.rolled_back += 1;
                }
                Err(error) => {
                    self.journal.entries[index].rollback_status = RollbackStatus::RollbackFailed;
                    self.journal.entries[index].external_change = Some(error.to_string());
                    outcome.failed += 1;
                }
            }
            self.persist()?;
        }
        if outcome.failed > 0 {
            Err(AppError::Conflict(format!(
                "{} owned paths failed to roll back",
                outcome.failed
            )))
        } else {
            Ok(outcome)
        }
    }

    pub fn restore_preview(&self) -> Result<RestorePreview, AppError> {
        let mut paths = Vec::new();
        for entry in &self.journal.entries {
            if !entry.committed || entry.rollback_status == RollbackStatus::RolledBack {
                continue;
            }
            let current_sha256 = file_state_hash(&entry.destination_path)?;
            paths.push(RestorePreviewPath {
                path: entry.destination_path.clone(),
                action: entry.action,
                safe_to_restore: current_sha256 == entry.committed_sha256,
                current_sha256,
                expected_committed_sha256: entry.committed_sha256.clone(),
            });
        }
        Ok(RestorePreview { paths })
    }

    pub fn journal(&self) -> &TransactionJournal {
        &self.journal
    }

    pub fn run_root(&self) -> &Path {
        &self.run_root
    }

    pub fn journal_path(&self) -> &Path {
        &self.journal_path
    }

    fn commit_entry(&mut self, index: usize) -> Result<(), AppError> {
        let entry = &self.journal.entries[index];
        let current = file_state_hash(&entry.destination_path)?;
        if current != entry.precondition_sha256 {
            return Err(AppError::Conflict(format!(
                "journal precondition mismatch at {}",
                entry.destination_path.display()
            )));
        }
        match entry.action {
            FileAction::Create | FileAction::Update => atomic_write(
                &entry.destination_path,
                entry
                    .intended_bytes
                    .as_deref()
                    .expect("write entry has intended bytes"),
                &self.options.run_id,
                index,
            )?,
            FileAction::Delete => std::fs::remove_file(&entry.destination_path)
                .map_err(|error| AppError::io(&entry.destination_path, error))?,
        }
        let committed_sha256 = file_state_hash(&entry.destination_path)?;
        let intended_sha256 = entry.intended_sha256.clone();
        let entry = &mut self.journal.entries[index];
        entry.committed = true;
        entry.committed_sha256 = committed_sha256.clone();
        if committed_sha256 != intended_sha256 {
            return Err(AppError::Conflict(format!(
                "committed hash mismatch at {}",
                entry.destination_path.display()
            )));
        }
        Ok(())
    }

    fn persist(&self) -> Result<(), AppError> {
        persist_json(&self.journal_path, &self.journal, &self.options.run_id)
    }
}

pub fn sha256_file(path: &Path) -> Result<String, AppError> {
    let mut file = File::open(path).map_err(|error| AppError::io(path, error))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| AppError::io(path, error))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn create_backup(destination_root: &Path, backup_path: &Path) -> Result<BackupEvidence, AppError> {
    let filament = destination_root.join("filament");
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(backup_path)
        .map_err(|error| AppError::io(backup_path, error))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut files = Vec::new();
    if filament.is_dir() {
        for entry in WalkDir::new(&filament).follow_links(false) {
            let entry = entry.map_err(|error| {
                AppError::InvalidProfile(format!("backup traversal failed: {error}"))
            })?;
            if entry.file_type().is_symlink() {
                return Err(AppError::UnsafePath(entry.path().to_path_buf()));
            }
            if entry.file_type().is_file() {
                files.push(entry.into_path());
            }
        }
    }
    files.sort();
    for path in &files {
        let relative = path
            .strip_prefix(destination_root)
            .map_err(|_| AppError::UnsafePath(path.clone()))?;
        let archive_name = relative.to_string_lossy().replace('\\', "/");
        writer
            .start_file(archive_name, options)
            .map_err(|error| AppError::InvalidProfile(format!("backup ZIP failed: {error}")))?;
        let bytes = std::fs::read(path).map_err(|error| AppError::io(path, error))?;
        writer
            .write_all(&bytes)
            .map_err(|error| AppError::io(backup_path, error))?;
    }
    let file = writer
        .finish()
        .map_err(|error| AppError::InvalidProfile(format!("backup ZIP failed: {error}")))?;
    file.sync_all()
        .map_err(|error| AppError::io(backup_path, error))?;
    let size = file
        .metadata()
        .map_err(|error| AppError::io(backup_path, error))?
        .len();
    Ok(BackupEvidence {
        path: backup_path.to_path_buf(),
        sha256: sha256_file(backup_path)?,
        size,
        file_count: files.len(),
    })
}

fn prepare_run_root(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        if !path.is_dir() {
            return Err(AppError::Conflict(format!(
                "run path is not a directory: {}",
                path.display()
            )));
        }
        if std::fs::read_dir(path)
            .map_err(|error| AppError::io(path, error))?
            .next()
            .transpose()
            .map_err(|error| AppError::io(path, error))?
            .is_some()
        {
            return Err(AppError::Conflict(format!(
                "run directory is not empty: {}",
                path.display()
            )));
        }
    } else {
        std::fs::create_dir_all(path).map_err(|error| AppError::io(path, error))?;
    }
    Ok(())
}

fn validate_run_id(run_id: &str) -> Result<(), AppError> {
    if run_id.is_empty()
        || !run_id
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_'))
    {
        return Err(AppError::InvalidProfile("run id is invalid".to_owned()));
    }
    Ok(())
}

fn validate_relative_path(path: &Path) -> Result<(), AppError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AppError::UnsafePath(path.to_path_buf()));
    }
    Ok(())
}

fn guarded_destination(root: &Path, relative: &Path) -> Result<PathBuf, AppError> {
    validate_relative_path(relative)?;
    let destination = root.join(relative);
    let mut cursor = destination
        .parent()
        .ok_or_else(|| AppError::UnsafePath(destination.clone()))?;
    while !cursor.exists() {
        cursor = cursor
            .parent()
            .ok_or_else(|| AppError::UnsafePath(destination.clone()))?;
    }
    let canonical_parent = cursor
        .canonicalize()
        .map_err(|error| AppError::io(cursor, error))?;
    if !canonical_parent.starts_with(root) {
        return Err(AppError::UnsafePath(destination));
    }
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        if current.exists()
            && std::fs::symlink_metadata(&current)
                .map_err(|error| AppError::io(&current, error))?
                .file_type()
                .is_symlink()
        {
            return Err(AppError::UnsafePath(current));
        }
    }
    Ok(destination)
}

fn fold_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn read_optional_file(path: &Path) -> Result<Option<Vec<u8>>, AppError> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(AppError::io(path, error)),
    }
}

fn file_state_hash(path: &Path) -> Result<Option<String>, AppError> {
    read_optional_file(path).map(|bytes| bytes.as_deref().map(sha256_bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn atomic_write(
    destination: &Path,
    bytes: &[u8],
    run_id: &str,
    index: usize,
) -> Result<(), AppError> {
    let parent = destination
        .parent()
        .ok_or_else(|| AppError::UnsafePath(destination.to_path_buf()))?;
    std::fs::create_dir_all(parent).map_err(|error| AppError::io(parent, error))?;
    let temp = parent.join(format!(".bfm-{run_id}-{index}.tmp"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| AppError::io(&temp, error))?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        let _ = std::fs::remove_file(&temp);
        return Err(AppError::io(&temp, error));
    }
    drop(file);
    replace_path(&temp, destination)
}

#[cfg(not(target_os = "windows"))]
fn replace_path(temp: &Path, destination: &Path) -> Result<(), AppError> {
    std::fs::rename(temp, destination).map_err(|error| AppError::io(destination, error))
}

#[cfg(target_os = "windows")]
fn replace_path(temp: &Path, destination: &Path) -> Result<(), AppError> {
    if !destination.exists() {
        return std::fs::rename(temp, destination)
            .map_err(|error| AppError::io(destination, error));
    }
    let backup = temp.with_extension("previous");
    std::fs::rename(destination, &backup).map_err(|error| AppError::io(destination, error))?;
    if let Err(error) = std::fs::rename(temp, destination) {
        return match std::fs::rename(&backup, destination) {
            Ok(()) => Err(AppError::io(destination, error)),
            Err(restore_error) => Err(AppError::Conflict(format!(
                "replacement failed at {}; prior bytes remain at {} but could not be restored: {restore_error}",
                destination.display(),
                backup.display()
            ))),
        };
    }
    let _ = std::fs::remove_file(backup);
    Ok(())
}

fn remove_if_exists(path: &Path) -> Result<(), AppError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::io(path, error)),
    }
}

fn persist_json<T: Serialize>(path: &Path, value: &T, run_id: &str) -> Result<(), AppError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
    bytes.push(b'\n');
    let parent = path
        .parent()
        .ok_or_else(|| AppError::UnsafePath(path.to_path_buf()))?;
    std::fs::create_dir_all(parent).map_err(|error| AppError::io(parent, error))?;
    let temp = parent.join(format!(".{run_id}-journal.tmp"));
    if temp.exists() {
        std::fs::remove_file(&temp).map_err(|error| AppError::io(&temp, error))?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| AppError::io(&temp, error))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| AppError::io(&temp, error))?;
    drop(file);
    replace_path(&temp, path)
}
