use crate::AppError;
use crate::model::{ProfileId, SourceApp, SourceKind};
use crate::naming::{NamingContext, NamingTemplate};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    New,
    AlreadyMigrated,
    Incomplete,
    Conflicting,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationSource {
    pub id: ProfileId,
    pub name: String,
    pub vendor: String,
    pub material: String,
    pub family: String,
    pub variant: String,
    pub source_app: SourceApp,
    pub source_kind: SourceKind,
    pub compatible_printers: BTreeSet<String>,
    pub migration_status: MigrationStatus,
    pub settings_fingerprint: String,
    pub existing_filament_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetSelection {
    pub printer_id: String,
    pub printer_name: String,
    pub printer_code: String,
    pub nozzle: String,
    pub printer_preset_name: String,
    pub custom_unverified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRequest {
    pub sources: Vec<MigrationSource>,
    pub targets: Vec<TargetSelection>,
    pub preset_template: String,
    pub ams_template: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterQuery {
    pub search: String,
    pub source_apps: BTreeSet<SourceApp>,
    pub source_kinds: BTreeSet<SourceKind>,
    pub vendors: BTreeSet<String>,
    pub materials: BTreeSet<String>,
    pub families: BTreeSet<String>,
    pub variants: BTreeSet<String>,
    pub compatible_printers: BTreeSet<String>,
    pub migration_statuses: BTreeSet<MigrationStatus>,
    pub selected_only: bool,
}

impl FilterQuery {
    pub fn apply<'a>(
        &self,
        sources: &'a [MigrationSource],
        selected: &BTreeSet<ProfileId>,
    ) -> Vec<&'a MigrationSource> {
        let search = self.search.trim().to_ascii_lowercase();
        sources
            .iter()
            .filter(|source| !self.selected_only || selected.contains(&source.id))
            .filter(|source| {
                self.source_apps.is_empty() || self.source_apps.contains(&source.source_app)
            })
            .filter(|source| {
                self.source_kinds.is_empty() || self.source_kinds.contains(&source.source_kind)
            })
            .filter(|source| matches_text_facet(&self.vendors, &source.vendor))
            .filter(|source| matches_text_facet(&self.materials, &source.material))
            .filter(|source| matches_text_facet(&self.families, &source.family))
            .filter(|source| matches_text_facet(&self.variants, &source.variant))
            .filter(|source| {
                self.compatible_printers.is_empty()
                    || source.compatible_printers.iter().any(|printer| {
                        contains_case_insensitive(&self.compatible_printers, printer)
                    })
            })
            .filter(|source| {
                self.migration_statuses.is_empty()
                    || self.migration_statuses.contains(&source.migration_status)
            })
            .filter(|source| {
                search.is_empty()
                    || [
                        source.name.as_str(),
                        source.vendor.as_str(),
                        source.material.as_str(),
                        source.family.as_str(),
                        source.variant.as_str(),
                    ]
                    .iter()
                    .any(|value| value.to_ascii_lowercase().contains(&search))
            })
            .collect()
    }
}

fn matches_text_facet(values: &BTreeSet<String>, candidate: &str) -> bool {
    values.is_empty() || contains_case_insensitive(values, candidate)
}

fn contains_case_insensitive(values: &BTreeSet<String>, candidate: &str) -> bool {
    values
        .iter()
        .any(|value| value.eq_ignore_ascii_case(candidate))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IdentityFingerprint {
    pub name: String,
    pub printer: String,
    pub nozzle: String,
}

impl IdentityFingerprint {
    pub fn new(name: &str, printer: &str, nozzle: &str) -> Self {
        Self {
            name: normalize_identity(name),
            printer: normalize_identity(printer),
            nozzle: normalize_identity(nozzle),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExistingDestination {
    pub name: String,
    pub settings_fingerprint: String,
    pub filament_id: String,
    pub printer_preset_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct DestinationIndex {
    existing: Vec<ExistingDestination>,
}

impl DestinationIndex {
    pub fn from_existing(items: impl IntoIterator<Item = ExistingDestination>) -> Self {
        let mut existing: Vec<_> = items.into_iter().collect();
        existing.sort_by(|left, right| {
            (&left.name, &left.printer_preset_name, &left.filament_id).cmp(&(
                &right.name,
                &right.printer_preset_name,
                &right.filament_id,
            ))
        });
        Self { existing }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanAction {
    Create,
    AddTarget,
    Update,
    Rename,
    Replace,
    Skip,
    Block,
}

impl PlanAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::AddTarget => "add_target",
            Self::Update => "update",
            Self::Rename => "rename",
            Self::Replace => "replace",
            Self::Skip => "skip",
            Self::Block => "block",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    CaseOnlyName,
    IdentityCollision,
    SettingsMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    pub kind: ConflictKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanOperation {
    pub id: String,
    pub source_id: ProfileId,
    pub source_name: String,
    pub preset_name: String,
    pub ams_name: String,
    pub filament_id: String,
    pub printer_id: String,
    pub printer_name: String,
    pub printer_preset_name: String,
    pub nozzle: String,
    pub custom_unverified: bool,
    pub source_settings_fingerprint: String,
    pub precondition_fingerprint: Option<String>,
    pub identity_fingerprint: IdentityFingerprint,
    pub action: PlanAction,
    pub conflict: Option<Conflict>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub id: String,
    pub operations: Vec<PlanOperation>,
}

pub struct Planner;

impl Planner {
    pub fn build(
        request: &MigrationRequest,
        destinations: &DestinationIndex,
    ) -> Result<MigrationPlan, AppError> {
        if request.sources.is_empty() {
            return Err(AppError::InvalidProfile(
                "migration has no selected source profiles".to_owned(),
            ));
        }
        if request.targets.is_empty() {
            return Err(AppError::InvalidProfile(
                "migration has no selected target nozzles".to_owned(),
            ));
        }
        if request.user_id.trim().is_empty() {
            return Err(AppError::InvalidProfile(
                "destination user id is empty".to_owned(),
            ));
        }
        let preset_template = NamingTemplate::parse(&request.preset_template)?;
        let ams_template = NamingTemplate::parse(&request.ams_template)?;
        let mut sources = request.sources.iter().collect::<Vec<_>>();
        sources.sort_by(|left, right| (&left.name, &left.id).cmp(&(&right.name, &right.id)));
        let mut targets = request.targets.iter().collect::<Vec<_>>();
        targets.sort_by(|left, right| {
            (&left.printer_name, &left.printer_id)
                .cmp(&(&right.printer_name, &right.printer_id))
                .then_with(|| numeric_cmp(&left.nozzle, &right.nozzle))
                .then_with(|| left.printer_preset_name.cmp(&right.printer_preset_name))
        });

        let mut operations = Vec::with_capacity(sources.len() * targets.len());
        for source in sources {
            for target in &targets {
                let context = naming_context(source, target);
                let preset_name = preset_template.render(&context)?;
                let ams_name = ams_template.render(&context)?;
                let filament_id = source
                    .existing_filament_id
                    .clone()
                    .unwrap_or_else(|| bambu_filament_id(&ams_name, &request.user_id));
                let identity_fingerprint =
                    IdentityFingerprint::new(&ams_name, &target.printer_code, &target.nozzle);
                let (action, conflict, precondition_fingerprint) = classify_destination(
                    destinations,
                    &ams_name,
                    &filament_id,
                    &target.printer_preset_name,
                    &source.settings_fingerprint,
                );
                let operation_key = serde_json::to_vec(&(
                    &source.id,
                    &target.printer_id,
                    &target.nozzle,
                    &preset_name,
                    &ams_name,
                    &filament_id,
                    &source.settings_fingerprint,
                ))
                .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
                operations.push(PlanOperation {
                    id: sha256_hex(&operation_key),
                    source_id: source.id.clone(),
                    source_name: source.name.clone(),
                    preset_name,
                    ams_name,
                    filament_id,
                    printer_id: target.printer_id.clone(),
                    printer_name: target.printer_name.clone(),
                    printer_preset_name: target.printer_preset_name.clone(),
                    nozzle: target.nozzle.clone(),
                    custom_unverified: target.custom_unverified,
                    source_settings_fingerprint: source.settings_fingerprint.clone(),
                    precondition_fingerprint,
                    identity_fingerprint,
                    action,
                    conflict,
                });
            }
        }
        reject_duplicate_outputs(&mut operations);
        reject_generated_id_collisions(&mut operations);
        let canonical = serde_json::to_vec(&operations)
            .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
        Ok(MigrationPlan {
            id: sha256_hex(&canonical),
            operations,
        })
    }
}

fn naming_context(source: &MigrationSource, target: &TargetSelection) -> NamingContext {
    NamingContext::new(&source.name, &source.vendor, &source.material)
        .with("family", source.family.clone())
        .with("variant", source.variant.clone())
        .with("source_app", source_app_label(source.source_app))
        .with("source_kind", source_kind_label(source.source_kind))
        .with("printer", target.printer_name.clone())
        .with("printer_code", target.printer_code.clone())
        .with("nozzle", target.nozzle.clone())
}

fn classify_destination(
    destinations: &DestinationIndex,
    name: &str,
    filament_id: &str,
    printer_preset_name: &str,
    settings_fingerprint: &str,
) -> (PlanAction, Option<Conflict>, Option<String>) {
    if let Some(existing) = destinations.existing.iter().find(|item| {
        item.printer_preset_name == printer_preset_name && item.name.eq_ignore_ascii_case(name)
    }) {
        let precondition = Some(existing.settings_fingerprint.clone());
        if existing.name != name {
            return (
                PlanAction::Block,
                Some(Conflict {
                    kind: ConflictKind::CaseOnlyName,
                    message: format!("destination name differs only by case: {}", existing.name),
                }),
                precondition,
            );
        }
        if existing.filament_id != filament_id {
            return (
                PlanAction::Block,
                Some(Conflict {
                    kind: ConflictKind::IdentityCollision,
                    message: "destination name is already tied to another filament id".to_owned(),
                }),
                precondition,
            );
        }
        if existing.settings_fingerprint != settings_fingerprint {
            return (
                PlanAction::Block,
                Some(Conflict {
                    kind: ConflictKind::SettingsMismatch,
                    message: "destination identity has different effective settings".to_owned(),
                }),
                precondition,
            );
        }
        return (PlanAction::Skip, None, precondition);
    }

    let matching_identity = destinations
        .existing
        .iter()
        .find(|item| item.filament_id == filament_id || item.name.eq_ignore_ascii_case(name));
    if let Some(existing) = matching_identity {
        let precondition = Some(existing.settings_fingerprint.clone());
        if existing.name != name || existing.filament_id != filament_id {
            return (
                PlanAction::Block,
                Some(Conflict {
                    kind: ConflictKind::IdentityCollision,
                    message: "an overlapping destination identity already exists".to_owned(),
                }),
                precondition,
            );
        }
        if existing.settings_fingerprint != settings_fingerprint {
            return (
                PlanAction::Block,
                Some(Conflict {
                    kind: ConflictKind::SettingsMismatch,
                    message: "destination identity has different effective settings".to_owned(),
                }),
                precondition,
            );
        }
        return (PlanAction::AddTarget, None, precondition);
    }
    (PlanAction::Create, None, None)
}

fn reject_duplicate_outputs(operations: &mut [PlanOperation]) {
    for index in 0..operations.len() {
        let duplicate = operations[..index].iter().any(|other| {
            other.printer_preset_name == operations[index].printer_preset_name
                && (other
                    .preset_name
                    .eq_ignore_ascii_case(&operations[index].preset_name)
                    || other
                        .ams_name
                        .eq_ignore_ascii_case(&operations[index].ams_name))
        });
        if duplicate {
            operations[index].action = PlanAction::Block;
            operations[index].conflict = Some(Conflict {
                kind: ConflictKind::IdentityCollision,
                message: "multiple selected sources generate the same destination name".to_owned(),
            });
        }
    }
}

fn reject_generated_id_collisions(operations: &mut [PlanOperation]) {
    let mut by_id: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, operation) in operations.iter().enumerate() {
        by_id
            .entry(operation.filament_id.clone())
            .or_default()
            .push(index);
    }
    for indices in by_id.values() {
        let identities: BTreeSet<_> = indices
            .iter()
            .map(|index| {
                (
                    normalize_identity(&operations[*index].ams_name),
                    operations[*index].source_settings_fingerprint.as_str(),
                )
            })
            .collect();
        if identities.len() > 1 {
            for index in indices {
                operations[*index].action = PlanAction::Block;
                operations[*index].conflict = Some(Conflict {
                    kind: ConflictKind::IdentityCollision,
                    message: "generated filament id maps to multiple identities".to_owned(),
                });
            }
        }
    }
}

fn bambu_filament_id(display_name: &str, user_id: &str) -> String {
    let digest = md5::compute(format!("{display_name}@{user_id}"));
    format!("P{:x}", digest)[..8].to_owned()
}

fn sha256_hex(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn normalize_identity(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn numeric_cmp(left: &str, right: &str) -> Ordering {
    match (left.parse::<f64>(), right.parse::<f64>()) {
        (Ok(left), Ok(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        _ => left.cmp(right),
    }
}

fn source_app_label(value: SourceApp) -> &'static str {
    match value {
        SourceApp::OrcaSlicer => "OrcaSlicer",
        SourceApp::BambuStudio => "Bambu Studio",
    }
}

fn source_kind_label(value: SourceKind) -> &'static str {
    match value {
        SourceKind::FactorySystem => "Factory/System",
        SourceKind::UserCustom => "User/Custom",
    }
}
