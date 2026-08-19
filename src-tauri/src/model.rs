use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfileId(pub String);

impl ProfileId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceApp {
    OrcaSlicer,
    BambuStudio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    FactorySystem,
    UserCustom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    CreatedLocal,
    LoadedByBambu,
    CloudIdAssigned,
    AmsVerified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    Planned,
    CreatedLocal,
    UpdatedLocal,
    SkippedExisting,
    SkippedByUser,
    BlockedInvalidSource,
    BlockedMissingParent,
    BlockedUnsupportedSchema,
    BlockedConflict,
    RolledBack,
    CreatedLocalUnsynchronized,
    LoadedByBambu,
    CloudIdAssigned,
    AmsVerified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountEligibility {
    Eligible,
    Empty,
    Stale,
    DefaultLocal,
    Backup,
    Unsupported,
}

impl AccountEligibility {
    pub fn writable_by_default(self) -> bool {
        self == Self::Eligible
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrinterPresetKind {
    Official,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceProfileSummary {
    pub id: ProfileId,
    pub name: String,
    pub source_app: SourceApp,
    pub source_kind: SourceKind,
    pub vendor: String,
    pub material: String,
    pub family: String,
    pub variant: String,
    pub path: PathBuf,
    pub compatible_printers: BTreeSet<String>,
    pub warnings: Vec<String>,
    pub blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NozzleTarget {
    pub id: String,
    pub diameter: String,
    pub printer_preset_name: String,
    pub selected: bool,
    pub supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrinterTarget {
    pub id: String,
    pub name: String,
    pub code: String,
    pub kind: PrinterPresetKind,
    pub verified: bool,
    pub extruder_variants: Vec<String>,
    pub nozzles: Vec<NozzleTarget>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_levels_use_stable_snake_case_values() {
        assert_eq!(
            serde_json::to_string(&EvidenceLevel::CloudIdAssigned).unwrap(),
            "\"cloud_id_assigned\""
        );
    }

    #[test]
    fn only_eligible_accounts_are_writable_by_default() {
        assert!(AccountEligibility::Eligible.writable_by_default());
        assert!(!AccountEligibility::Empty.writable_by_default());
    }
}
