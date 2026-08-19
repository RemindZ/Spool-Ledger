use crate::AppError;
use crate::transaction::{BackupEvidence, FileAction, Transaction};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptState {
    Planned,
    Committed,
    RolledBack,
    PartialRollback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptCounts {
    pub creates: usize,
    pub updates: usize,
    pub deletes: usize,
    pub committed: usize,
    pub rolled_back: usize,
    pub external_conflicts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunReceipt {
    pub version: u32,
    pub run_id: String,
    pub plan_id: String,
    pub created_at: i64,
    pub destination_root: PathBuf,
    pub backup: BackupEvidence,
    pub journal_path: PathBuf,
    pub state: ReceiptState,
    pub counts: ReceiptCounts,
}

impl RunReceipt {
    pub fn from_transaction(
        transaction: &Transaction,
        state: ReceiptState,
    ) -> Result<Self, AppError> {
        let journal = transaction.journal();
        let mut counts = ReceiptCounts {
            creates: 0,
            updates: 0,
            deletes: 0,
            committed: 0,
            rolled_back: 0,
            external_conflicts: 0,
        };
        for entry in &journal.entries {
            match entry.action {
                FileAction::Create => counts.creates += 1,
                FileAction::Update => counts.updates += 1,
                FileAction::Delete => counts.deletes += 1,
            }
            counts.committed += usize::from(entry.committed);
            counts.rolled_back += usize::from(matches!(
                entry.rollback_status,
                crate::transaction::RollbackStatus::RolledBack
            ));
            counts.external_conflicts += usize::from(matches!(
                entry.rollback_status,
                crate::transaction::RollbackStatus::ExternalChange
            ));
        }
        Ok(Self {
            version: 1,
            run_id: journal.run_id.clone(),
            plan_id: journal.plan_id.clone(),
            created_at: journal.created_at,
            destination_root: journal.destination_root.clone(),
            backup: journal.backup.clone(),
            journal_path: transaction.journal_path().to_path_buf(),
            state,
            counts,
        })
    }

    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
        bytes.push(b'\n');
        std::fs::write(path, bytes).map_err(|error| AppError::io(path, error))
    }

    pub fn load(path: &Path) -> Result<Self, AppError> {
        let bytes = std::fs::read(path).map_err(|error| AppError::io(path, error))?;
        serde_json::from_slice(&bytes).map_err(|source| AppError::Json {
            path: path.to_path_buf(),
            source,
        })
    }
}
