use bambu_filament_migrator::planner::MigrationPlan;
use bambu_filament_migrator::receipt::{ReceiptState, RunReceipt};
use bambu_filament_migrator::transaction::{
    FileAction, RollbackStatus, Transaction, TransactionOptions,
};
use bambu_filament_migrator::writer::{
    GeneratedArtifact, GeneratedArtifactKind, StagedDeletion, StagedRun,
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn plan() -> MigrationPlan {
    MigrationPlan {
        id: "plan-001".to_owned(),
        operations: Vec::new(),
    }
}

fn staged(root: &Path, writes: &[(&str, &[u8])], deletes: &[(&str, &[u8])]) -> StagedRun {
    std::fs::create_dir_all(root).unwrap();
    let mut artifacts = Vec::new();
    for (relative, bytes) in writes {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        artifacts.push(GeneratedArtifact {
            kind: GeneratedArtifactKind::CustomProfile,
            operation_ids: vec![format!("write:{relative}")],
            relative_path: PathBuf::from(relative),
            sha256: hash(bytes),
            size: bytes.len() as u64,
        });
    }
    StagedRun {
        root: root.to_path_buf(),
        artifacts,
        deletions: deletes
            .iter()
            .map(|(relative, expected)| StagedDeletion {
                operation_ids: vec![format!("delete:{relative}")],
                relative_path: PathBuf::from(relative),
                expected_sha256: hash(expected),
            })
            .collect(),
    }
}

fn options() -> TransactionOptions {
    TransactionOptions {
        run_id: "run-001".to_owned(),
        created_at: 1_787_140_000,
    }
}

fn preflight<'a>(staged: &'a StagedRun, destination: &'a Path, run_root: &'a Path) -> Transaction {
    Transaction::preflight(&plan(), staged, destination, run_root, options()).unwrap()
}

#[test]
fn commit_creates_and_updates_files_with_durable_journal() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("account");
    let stage_root = temp.path().join("stage");
    let run_root = temp.path().join("run");
    std::fs::create_dir_all(destination.join("filament")).unwrap();
    std::fs::write(destination.join("filament/existing.json"), b"old").unwrap();
    let staged = staged(
        &stage_root,
        &[
            ("filament/new.json", b"new"),
            ("filament/existing.json", b"updated"),
        ],
        &[],
    );
    let mut transaction = preflight(&staged, &destination, &run_root);
    let outcome = transaction.commit().unwrap();

    assert_eq!(outcome.committed, 2);
    assert_eq!(
        std::fs::read(destination.join("filament/new.json")).unwrap(),
        b"new"
    );
    assert_eq!(
        std::fs::read(destination.join("filament/existing.json")).unwrap(),
        b"updated"
    );
    assert!(run_root.join("journal.json").is_file());
    assert!(
        transaction
            .journal()
            .entries
            .iter()
            .all(|entry| entry.committed_sha256.is_some())
    );
}

#[test]
fn precondition_failure_after_partial_commit_rolls_back_only_prior_entries() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("account");
    let stage_root = temp.path().join("stage");
    let run_root = temp.path().join("run");
    std::fs::create_dir_all(destination.join("filament")).unwrap();
    std::fs::write(destination.join("filament/b.json"), b"before").unwrap();
    let staged = staged(
        &stage_root,
        &[
            ("filament/a.json", b"created"),
            ("filament/b.json", b"planned"),
        ],
        &[],
    );
    let mut transaction = preflight(&staged, &destination, &run_root);
    std::fs::write(destination.join("filament/b.json"), b"external").unwrap();
    let error = transaction.commit().unwrap_err();

    assert!(error.to_string().contains("precondition"));
    assert!(!destination.join("filament/a.json").exists());
    assert_eq!(
        std::fs::read(destination.join("filament/b.json")).unwrap(),
        b"external"
    );
    assert_eq!(
        transaction.journal().entries[0].rollback_status,
        RollbackStatus::RolledBack
    );
    assert_eq!(
        transaction.journal().entries[1].rollback_status,
        RollbackStatus::NotRequired
    );
}

#[test]
fn rollback_restores_updated_and_deleted_files_but_keeps_unrelated_files() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("account");
    let stage_root = temp.path().join("stage");
    let run_root = temp.path().join("run");
    std::fs::create_dir_all(destination.join("filament")).unwrap();
    std::fs::write(destination.join("filament/update.json"), b"old").unwrap();
    std::fs::write(destination.join("filament/delete.info"), b"delete-me").unwrap();
    let staged = staged(
        &stage_root,
        &[("filament/update.json", b"new")],
        &[("filament/delete.info", b"delete-me")],
    );
    let mut transaction = preflight(&staged, &destination, &run_root);
    let outcome = transaction.commit().unwrap();
    assert_eq!(outcome.committed, 2);
    assert!(!destination.join("filament/delete.info").exists());
    std::fs::write(destination.join("filament/unrelated.json"), b"later").unwrap();

    let rollback = transaction.rollback_owned().unwrap();
    assert_eq!(rollback.rolled_back, 2);
    assert_eq!(rollback.external_conflicts, 0);
    assert_eq!(
        std::fs::read(destination.join("filament/update.json")).unwrap(),
        b"old"
    );
    assert_eq!(
        std::fs::read(destination.join("filament/delete.info")).unwrap(),
        b"delete-me"
    );
    assert_eq!(
        std::fs::read(destination.join("filament/unrelated.json")).unwrap(),
        b"later"
    );
    assert!(
        transaction
            .journal()
            .entries
            .iter()
            .any(|entry| entry.action == FileAction::Delete)
    );
}

#[test]
fn rollback_preserves_owned_path_changed_after_commit() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("account");
    let stage_root = temp.path().join("stage");
    let run_root = temp.path().join("run");
    std::fs::create_dir_all(destination.join("filament")).unwrap();
    let staged = staged(
        &stage_root,
        &[("filament/generated.json", b"generated")],
        &[],
    );
    let mut transaction = preflight(&staged, &destination, &run_root);
    transaction.commit().unwrap();
    std::fs::write(destination.join("filament/generated.json"), b"external").unwrap();

    let rollback = transaction.rollback_owned().unwrap();
    assert_eq!(rollback.rolled_back, 0);
    assert_eq!(rollback.external_conflicts, 1);
    assert_eq!(
        std::fs::read(destination.join("filament/generated.json")).unwrap(),
        b"external"
    );
    assert_eq!(
        transaction.journal().entries[0].rollback_status,
        RollbackStatus::ExternalChange
    );
}

#[test]
fn preflight_creates_checksum_verified_full_filament_backup() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("account");
    let stage_root = temp.path().join("stage");
    let run_root = temp.path().join("run");
    std::fs::create_dir_all(destination.join("filament/base")).unwrap();
    std::fs::write(destination.join("filament/base/existing.json"), b"profile").unwrap();
    let staged = staged(&stage_root, &[("filament/new.json", b"new")], &[]);
    let transaction = preflight(&staged, &destination, &run_root);

    assert!(transaction.journal().backup.path.is_file());
    assert_eq!(
        transaction.journal().backup.sha256,
        bambu_filament_migrator::transaction::sha256_file(&transaction.journal().backup.path)
            .unwrap()
    );
    let file = std::fs::File::open(&transaction.journal().backup.path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut backed_up = archive.by_name("filament/base/existing.json").unwrap();
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut backed_up, &mut bytes).unwrap();
    assert_eq!(bytes, b"profile");
}

#[test]
fn persisted_transaction_can_be_reloaded_and_rolled_back() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("account");
    let stage_root = temp.path().join("stage");
    let run_root = temp.path().join("run");
    std::fs::create_dir_all(destination.join("filament")).unwrap();
    std::fs::write(destination.join("filament/existing.json"), b"old").unwrap();
    let staged = staged(&stage_root, &[("filament/existing.json", b"new")], &[]);
    let mut transaction = preflight(&staged, &destination, &run_root);
    transaction.commit().unwrap();
    drop(transaction);

    let mut loaded = Transaction::load(&run_root).unwrap();
    assert_eq!(loaded.restore_preview().unwrap().paths.len(), 1);
    loaded.rollback_owned().unwrap();
    assert_eq!(
        std::fs::read(destination.join("filament/existing.json")).unwrap(),
        b"old"
    );
}

#[test]
fn receipt_and_restore_preview_round_trip_without_applying_backup() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("account");
    let stage_root = temp.path().join("stage");
    let run_root = temp.path().join("run");
    std::fs::create_dir_all(destination.join("filament")).unwrap();
    std::fs::write(destination.join("filament/existing.json"), b"old").unwrap();
    let staged = staged(
        &stage_root,
        &[
            ("filament/existing.json", b"new"),
            ("filament/create.json", b"create"),
        ],
        &[],
    );
    let mut transaction = preflight(&staged, &destination, &run_root);
    transaction.commit().unwrap();
    let preview = transaction.restore_preview().unwrap();
    assert_eq!(preview.paths.len(), 2);
    assert!(preview.paths.iter().all(|item| item.safe_to_restore));

    let receipt = RunReceipt::from_transaction(&transaction, ReceiptState::Committed).unwrap();
    let path = run_root.join("receipt.json");
    receipt.save(&path).unwrap();
    let loaded = RunReceipt::load(&path).unwrap();
    assert_eq!(loaded, receipt);
    assert_eq!(
        std::fs::read(destination.join("filament/existing.json")).unwrap(),
        b"new"
    );
}
