use crate::AppError;
use crate::discovery::{AccountRoot, DiscoveryService, DiscoverySnapshot, inspect_account};
use crate::model::{
    AccountEligibility, EvidenceLevel, PrinterTarget, ProfileId, SourceApp, SourceKind,
};
use crate::naming::{NamingContext, NamingTemplate, ReplacementRuleSpec};
use crate::planner::{
    Conflict, ConflictChoice, ConflictKind, DestinationIndex, MaterialFingerprintIndex,
    MigrationPlan, MigrationRequest, MigrationSource, MigrationStatus, NamingOptions, PlanAction,
    Planner, TargetSelection,
};
use crate::profiles::InfoSidecar;
use crate::receipt::{AmsVerificationEvidence, ReceiptState, RunReceipt};
use crate::resolver::{CatalogRoot, EffectiveProfile, ProfileCatalog};
use crate::sync::{
    Clock, FsSyncRuntime, ProcessBackend, ProcessController, SyncExpectation, SyncMonitor,
    SyncResult, SyncRuntime, SystemClock, SystemProcessBackend, sha256_bytes,
};
use crate::targets::TargetCatalog;
use crate::transaction::{RestorePreview, RollbackOutcome, Transaction, TransactionOptions};
pub use crate::writer::OutputSelection;
use crate::writer::{
    BambuAdapter, ExpectedFileState, TargetProfileReference, Writer, WriterContext,
    effective_settings_fingerprint, generated_material_settings_fingerprint,
    material_settings_fingerprint,
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::ipc::{Channel, Response};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

const MANUAL_SOURCE_ROOTS_FILE: &str = "manual-source-roots-v1.json";
const CATALOG_PROGRESS_BATCH_SIZE: usize = 25;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManualSourceRootRecord {
    path: PathBuf,
    source_app: SourceApp,
    source_kind: SourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManualSourceRootFile {
    version: u32,
    roots: Vec<ManualSourceRootRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualSourceRootSummary {
    pub id: String,
    pub path: PathBuf,
    pub source_app: SourceApp,
    pub source_kind: SourceKind,
    pub active: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApprovedSourceRoot {
    pub id: String,
    pub path: PathBuf,
    pub source_app: SourceApp,
    pub source_kind: SourceKind,
}

#[derive(Debug, Clone)]
pub struct ApprovedTargetCatalog {
    pub id: String,
    pub manifest_path: PathBuf,
    pub profile_root: PathBuf,
    pub custom_machine_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ApprovedAccount {
    pub account: AccountRoot,
    pub bambu_executable: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub sources: Vec<ApprovedSourceRoot>,
    pub targets: Vec<ApprovedTargetCatalog>,
    pub accounts: Vec<ApprovedAccount>,
    pub data_root: PathBuf,
    pub process_close_timeout_ms: u64,
}

impl ServiceConfig {
    pub fn current() -> Result<(Self, DiscoverySnapshot), AppError> {
        let discovery = DiscoveryService::current()?.scan()?;
        let mut sources = Vec::new();
        register_system_source(&mut sources, &discovery.orca, SourceApp::OrcaSlicer);
        register_additional_system_sources(&mut sources, &discovery.orca, SourceApp::OrcaSlicer)?;
        register_system_source(&mut sources, &discovery.bambu, SourceApp::BambuStudio);
        for account in &discovery.orca_accounts {
            register_user_source(&mut sources, account, SourceApp::OrcaSlicer);
        }
        for account in &discovery.bambu_accounts {
            register_user_source(&mut sources, account, SourceApp::BambuStudio);
        }
        let targets = installed_system_profiles(
            &discovery.bambu.config_root,
            discovery.bambu.executable.as_deref(),
        )
        .map(|profiles| ApprovedTargetCatalog {
            id: "target:bambu:installed".to_owned(),
            manifest_path: profiles.manifest_path,
            profile_root: profiles.profile_root.join("filament"),
            custom_machine_root: None,
        })
        .into_iter()
        .collect();
        let accounts = discovery
            .bambu_accounts
            .iter()
            .cloned()
            .map(|account| ApprovedAccount {
                account,
                bambu_executable: discovery.bambu.executable.clone(),
            })
            .collect();
        let project =
            ProjectDirs::from("com", "Remindz", "Bambu Filament Migrator").ok_or_else(|| {
                AppError::InvalidProfile("application data directory unavailable".to_owned())
            })?;
        Ok((
            Self {
                sources,
                targets,
                accounts,
                data_root: project.data_local_dir().to_path_buf(),
                process_close_timeout_ms: 15_000,
            },
            discovery,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstalledSystemProfiles {
    manifest_path: PathBuf,
    profile_root: PathBuf,
}

fn installed_system_profiles(
    config_root: &Path,
    executable: Option<&Path>,
) -> Option<InstalledSystemProfiles> {
    let mut candidates = vec![config_root.join("system/BBL")];
    if let Some(parent) = executable.and_then(Path::parent) {
        candidates.push(parent.join("resources/profiles/BBL"));
        candidates.push(parent.join("../Resources/profiles/BBL"));
        candidates.push(parent.join("../share/bambu-studio/resources/profiles/BBL"));
    }
    candidates.into_iter().find_map(|profile_root| {
        if !profile_root.join("filament").is_dir() {
            return None;
        }
        let manifest_path = [
            Some(profile_root.join("BBL.json")),
            profile_root.parent().map(|parent| parent.join("BBL.json")),
        ]
        .into_iter()
        .flatten()
        .find(|path| path.is_file())?;
        Some(InstalledSystemProfiles {
            manifest_path: manifest_path.canonicalize().ok()?,
            profile_root: profile_root.canonicalize().ok()?,
        })
    })
}

fn register_system_source(
    sources: &mut Vec<ApprovedSourceRoot>,
    installation: &crate::discovery::Installation,
    source_app: SourceApp,
) {
    let Some(profiles) = installed_system_profiles(
        &installation.config_root,
        installation.executable.as_deref(),
    ) else {
        return;
    };
    sources.push(ApprovedSourceRoot {
        id: format!("source:{}:system", source_app_id(source_app)),
        path: profiles.profile_root.join("filament"),
        source_app,
        source_kind: SourceKind::FactorySystem,
    });
}

fn register_additional_system_sources(
    sources: &mut Vec<ApprovedSourceRoot>,
    installation: &crate::discovery::Installation,
    source_app: SourceApp,
) -> Result<(), AppError> {
    let mut seen_bundles: BTreeSet<String> = sources
        .iter()
        .filter(|root| root.source_app == source_app)
        .filter_map(|root| root.path.parent())
        .filter_map(Path::file_name)
        .filter_map(|name| name.to_str())
        .map(str::to_ascii_lowercase)
        .collect();
    let mut parents = vec![(installation.config_root.join("system"), false)];
    if let Some(parent) = installation.executable.as_deref().and_then(Path::parent) {
        parents.push((parent.join("resources/profiles"), true));
        parents.push((parent.join("../Resources/profiles"), true));
        parents.push((
            parent.join("../share/bambu-studio/resources/profiles"),
            true,
        ));
    }
    for (parent, library_only) in parents {
        if !parent.is_dir() {
            continue;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&parent)
            .map_err(|error| AppError::io(&parent, error))?
            .collect::<Result<_, _>>()
            .map_err(|error| AppError::io(&parent, error))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            if !entry
                .file_type()
                .map_err(|error| AppError::io(entry.path(), error))?
                .is_dir()
            {
                continue;
            }
            let bundle = entry.file_name().to_string_lossy().to_string();
            let bundle_key = bundle.to_ascii_lowercase();
            if library_only && bundle_key != "orcafilamentlibrary" {
                continue;
            }
            let profile_root = entry.path().join("filament");
            if !profile_root.is_dir() || !seen_bundles.insert(bundle_key.clone()) {
                continue;
            }
            let profile_root = profile_root
                .canonicalize()
                .map_err(|error| AppError::io(&profile_root, error))?;
            let fingerprint = sha256_bytes(bundle_key.as_bytes());
            sources.push(ApprovedSourceRoot {
                id: format!(
                    "source:{}:system:{}",
                    source_app_id(source_app),
                    &fingerprint[..16]
                ),
                path: profile_root,
                source_app,
                source_kind: SourceKind::FactorySystem,
            });
        }
    }
    Ok(())
}

fn register_user_source(
    sources: &mut Vec<ApprovedSourceRoot>,
    account: &AccountRoot,
    source_app: SourceApp,
) {
    let path = account.path.join("filament");
    if path.is_dir() {
        sources.push(ApprovedSourceRoot {
            id: format!("source:{}:user:{}", source_app_id(source_app), account.id),
            path,
            source_app,
            source_kind: SourceKind::UserCustom,
        });
    }
}

fn source_app_id(value: SourceApp) -> &'static str {
    match value {
        SourceApp::OrcaSlicer => "orca",
        SourceApp::BambuStudio => "bambu",
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

fn source_kind_id(value: SourceKind) -> &'static str {
    match value {
        SourceKind::FactorySystem => "system",
        SourceKind::UserCustom => "user",
    }
}

fn source_profile_id(source_app: SourceApp, name: &str) -> ProfileId {
    ProfileId::new(format!("profile:{}:{name}", source_app_id(source_app)))
}

fn manual_source_id(record: &ManualSourceRootRecord) -> String {
    let fingerprint = sha256_bytes(
        format!(
            "{}\n{}\n{}",
            record.path.to_string_lossy(),
            source_app_id(record.source_app),
            source_kind_id(record.source_kind)
        )
        .as_bytes(),
    );
    format!(
        "source:manual:{}:{}",
        source_app_id(record.source_app),
        &fingerprint[..16]
    )
}

fn approved_manual_source(
    record: &ManualSourceRootRecord,
) -> Result<(ManualSourceRootRecord, ApprovedSourceRoot), AppError> {
    let selected = record
        .path
        .canonicalize()
        .map_err(|error| AppError::io(&record.path, error))?;
    let profile_root = if selected.join("filament").is_dir() {
        selected.join("filament")
    } else {
        selected
    };
    let profile_root = profile_root
        .canonicalize()
        .map_err(|error| AppError::io(&profile_root, error))?;
    let catalog_root = match record.source_kind {
        SourceKind::FactorySystem => CatalogRoot::system(&profile_root, record.source_app),
        SourceKind::UserCustom => CatalogRoot::user(&profile_root, record.source_app),
    };
    let catalog = ProfileCatalog::load_roots(&[catalog_root])?;
    if catalog.selectable_profiles().is_empty() {
        return Err(AppError::InvalidProfile(format!(
            "manual source folder has no selectable filament profiles: {}",
            profile_root.display()
        )));
    }
    let canonical = ManualSourceRootRecord {
        path: profile_root.clone(),
        source_app: record.source_app,
        source_kind: record.source_kind,
    };
    Ok((
        canonical.clone(),
        ApprovedSourceRoot {
            id: manual_source_id(&canonical),
            path: profile_root,
            source_app: canonical.source_app,
            source_kind: canonical.source_kind,
        },
    ))
}

fn load_manual_source_roots(data_root: &Path) -> Result<Vec<ManualSourceRootRecord>, AppError> {
    let path = data_root.join(MANUAL_SOURCE_ROOTS_FILE);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(AppError::io(&path, error)),
    };
    let file: ManualSourceRootFile =
        serde_json::from_slice(&bytes).map_err(|source| AppError::Json {
            path: path.clone(),
            source,
        })?;
    if file.version != 1 {
        return Err(AppError::InvalidProfile(format!(
            "unsupported manual source file version: {}",
            file.version
        )));
    }
    Ok(file.roots)
}

fn save_manual_source_roots(
    data_root: &Path,
    roots: &[ManualSourceRootRecord],
) -> Result<(), AppError> {
    std::fs::create_dir_all(data_root).map_err(|error| AppError::io(data_root, error))?;
    let path = data_root.join(MANUAL_SOURCE_ROOTS_FILE);
    let mut bytes = serde_json::to_vec_pretty(&ManualSourceRootFile {
        version: 1,
        roots: roots.to_vec(),
    })
    .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
    bytes.push(b'\n');
    std::fs::write(&path, bytes).map_err(|error| AppError::io(&path, error))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub source_root_ids: Vec<String>,
    pub manual_source_roots: Vec<ManualSourceRootSummary>,
    pub target_catalog_ids: Vec<String>,
    pub accounts: Vec<AccountRoot>,
    pub orca_slicer_detected: bool,
    pub bambu_studio_detected: bool,
    pub platform: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogSourcesRequest {
    pub root_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManualSourceFolderRequest {
    pub source_app: SourceApp,
    pub source_kind: SourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoveManualSourceFolderRequest {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogSource {
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
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogProgressPhase {
    Loading,
    Resolving,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogProgressEvent {
    pub phase: CatalogProgressPhase,
    pub processed: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCatalogResponse {
    pub catalog_id: String,
    pub sources: Vec<CatalogSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogTargetsRequest {
    pub catalog_id: String,
    pub show_custom: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetCatalogResponse {
    pub catalog_id: String,
    pub printers: Vec<PrinterTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NozzleSelection {
    pub printer_id: String,
    pub diameters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildPlanRequest {
    pub source_catalog_id: String,
    pub source_ids: Vec<String>,
    pub target_catalog_id: String,
    pub nozzles: Vec<NozzleSelection>,
    pub destination_account_id: String,
    pub preset_template: String,
    pub ams_template: String,
    #[serde(default)]
    pub outputs: OutputSelection,
    #[serde(default)]
    pub naming: NamingOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetTemplateDecisionAction {
    UseInstalled,
    UseSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetTemplateDecision {
    pub issue_id: String,
    pub action: TargetTemplateDecisionAction,
    pub profile_name: Option<String>,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvePlanDependenciesRequest {
    pub request: BuildPlanRequest,
    pub decisions: Vec<TargetTemplateDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetTemplateAffectedSource {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetTemplateInstalledCandidate {
    pub profile_name: String,
    pub recommended: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetTemplateSourceCandidate {
    pub source_id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetTemplateIssue {
    pub id: String,
    pub expected_name: String,
    pub material: String,
    pub printer_id: String,
    pub printer_name: String,
    pub nozzle: String,
    pub affected_sources: Vec<TargetTemplateAffectedSource>,
    pub installed_candidates: Vec<TargetTemplateInstalledCandidate>,
    pub source_candidates: Vec<TargetTemplateSourceCandidate>,
    pub diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BuildPlanResponse {
    Ready { plan: MigrationPlan },
    NeedsResolution { issues: Vec<TargetTemplateIssue> },
}

impl BuildPlanResponse {
    pub fn into_plan(self) -> Result<MigrationPlan, AppError> {
        match self {
            Self::Ready { plan } => Ok(plan),
            Self::NeedsResolution { issues } => Err(AppError::UnsupportedSchema(format!(
                "{} target template dependencies require resolution",
                issues.len()
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutePlanRequest {
    pub plan_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPhase {
    Validating,
    ClosingBambu,
    Staging,
    ValidatingOutput,
    BackingUp,
    Committing,
    WritingReceipt,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionProgressEvent {
    pub plan_id: String,
    pub phase: ExecutionPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncPhase {
    Launching,
    Monitoring,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SynchronizeRunRequest {
    pub run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncProgressEvent {
    pub run_id: String,
    pub phase: SyncPhase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalRunResult {
    pub run_id: String,
    pub plan_id: String,
    pub committed_files: usize,
    pub created_files: usize,
    pub updated_files: usize,
    pub deleted_files: usize,
    pub skipped_operations: usize,
    pub backup_sha256: String,
    pub backup_file_count: usize,
    pub receipt_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewNamesRequest {
    pub preset_template: String,
    pub ams_template: String,
    #[serde(default)]
    pub preset_rules: Vec<ReplacementRuleSpec>,
    #[serde(default)]
    pub ams_rules: Vec<ReplacementRuleSpec>,
    pub rows: Vec<PreviewNameRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewNameRow {
    pub source_name: String,
    pub vendor: String,
    pub material: String,
    pub family: String,
    pub variant: String,
    pub source_app: SourceApp,
    pub source_kind: SourceKind,
    pub printer: String,
    pub printer_code: String,
    pub nozzle: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewNameResult {
    pub preset_before: String,
    pub preset_name: String,
    pub ams_before: String,
    pub ams_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmsVerificationRequest {
    pub run_id: String,
    pub operation_ids: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AmsVerificationRecord {
    run_id: String,
    operation_ids: Vec<String>,
    note: String,
    recorded_at: i64,
    operator_verified: bool,
}

#[derive(Debug, Clone)]
struct SourceProfileLocator {
    source_app: SourceApp,
    profile_name: String,
}

struct CachedSources {
    roots: Vec<ApprovedSourceRoot>,
    sources: BTreeMap<String, MigrationSource>,
    catalogs: BTreeMap<SourceApp, ProfileCatalog>,
    profile_locators: BTreeMap<ProfileId, SourceProfileLocator>,
}

struct CachedTargets {
    approved: ApprovedTargetCatalog,
    catalog: TargetCatalog,
    profiles: ProfileCatalog,
}

#[derive(Clone)]
struct StoredPlan {
    plan: MigrationPlan,
    source_roots: Vec<ApprovedSourceRoot>,
    source_profile_locators: BTreeMap<ProfileId, SourceProfileLocator>,
    target: ApprovedTargetCatalog,
    target_profiles: BTreeMap<String, TargetProfileReference>,
    destination_preconditions: BTreeMap<PathBuf, ExpectedFileState>,
    outputs: OutputSelection,
    destination_account_id: String,
}

pub struct MigrationService {
    config: ServiceConfig,
    manual_source_roots: Vec<ManualSourceRootRecord>,
    source_catalogs: BTreeMap<String, CachedSources>,
    target_catalogs: BTreeMap<String, CachedTargets>,
    plans: BTreeMap<String, StoredPlan>,
    runs: BTreeMap<String, PathBuf>,
}

fn target_profile_matches_material(profile: &EffectiveProfile, material: &str) -> bool {
    profile
        .string_values("filament_type")
        .iter()
        .any(|value| value.eq_ignore_ascii_case(material))
}

fn target_profile_supports(
    profile: &EffectiveProfile,
    material: &str,
    target: &TargetSelection,
) -> bool {
    target_profile_matches_material(profile, material)
        && profile
            .string_values("compatible_printers")
            .contains(&target.printer_preset_name.as_str())
}

fn installed_target_candidates(
    profiles: &ProfileCatalog,
    material: &str,
    target: &TargetSelection,
) -> Vec<String> {
    let broad_suffix = format!("@BBL {}", target.printer_code);
    let nozzle_suffix = format!("@BBL {} {} nozzle", target.printer_code, target.nozzle);
    let mut candidates: Vec<_> = profiles
        .selectable_profiles()
        .into_iter()
        .filter(|record| {
            record.name.ends_with(&broad_suffix) || record.name.ends_with(&nozzle_suffix)
        })
        .filter_map(|record| {
            profiles
                .resolve_name(&record.name)
                .ok()
                .filter(|profile| target_profile_supports(profile, material, target))
                .map(|_| record.name.clone())
        })
        .collect();
    candidates.sort_by_key(|name| {
        (
            !name.ends_with(&nozzle_suffix),
            !name.starts_with("Bambu "),
            name.clone(),
        )
    });
    candidates.dedup();
    candidates
}

fn source_target_candidates(
    cache: &CachedSources,
    material: &str,
    target: &TargetSelection,
    adapter: &BambuAdapter,
) -> Vec<TargetTemplateSourceCandidate> {
    let mut candidates: Vec<_> = cache
        .sources
        .values()
        .filter(|source| {
            source.source_app == SourceApp::BambuStudio
                && source.source_kind == SourceKind::UserCustom
                && source.material.eq_ignore_ascii_case(material)
                && source
                    .compatible_printers
                    .contains(&target.printer_preset_name)
        })
        .filter_map(|source| {
            let locator = cache.profile_locators.get(&source.id)?;
            let profile = cache
                .catalogs
                .get(&locator.source_app)?
                .resolve_name(&locator.profile_name)
                .ok()?;
            if !target_profile_supports(&profile, material, target)
                || adapter.validate_target_profile(&profile).is_err()
            {
                return None;
            }
            Some(TargetTemplateSourceCandidate {
                source_id: source.id.0.clone(),
                name: source.name.clone(),
            })
        })
        .collect();
    candidates.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    candidates.dedup_by(|left, right| left.source_id == right.source_id);
    candidates
}

impl MigrationService {
    pub fn new(mut config: ServiceConfig) -> Result<Self, AppError> {
        let mut manual_source_roots = load_manual_source_roots(&config.data_root)?;
        for record in &mut manual_source_roots {
            let Ok((canonical, approved)) = approved_manual_source(record) else {
                continue;
            };
            *record = canonical;
            let duplicate = config.sources.iter().any(|root| {
                root.path
                    .canonicalize()
                    .is_ok_and(|path| path == approved.path)
                    && root.source_app == approved.source_app
                    && root.source_kind == approved.source_kind
            });
            if !duplicate {
                config.sources.push(approved);
            }
        }
        canonicalize_config(&mut config)?;
        Ok(Self {
            config,
            manual_source_roots,
            source_catalogs: BTreeMap::new(),
            target_catalogs: BTreeMap::new(),
            plans: BTreeMap::new(),
            runs: BTreeMap::new(),
        })
    }

    pub fn add_manual_source_folder(
        &mut self,
        path: &Path,
        source_app: SourceApp,
        source_kind: SourceKind,
    ) -> Result<String, AppError> {
        let (record, approved) = approved_manual_source(&ManualSourceRootRecord {
            path: path.to_path_buf(),
            source_app,
            source_kind,
        })?;
        if let Some(existing) = self.config.sources.iter().find(|root| {
            root.path == approved.path
                && root.source_app == source_app
                && root.source_kind == source_kind
                && !root.id.starts_with("source:manual:")
        }) {
            return Err(AppError::Conflict(format!(
                "source folder is already discovered as {}",
                existing.id
            )));
        }
        let id = approved.id.clone();
        if self
            .manual_source_roots
            .iter()
            .any(|item| manual_source_id(item) == id)
        {
            return Ok(id);
        }
        let mut records = self.manual_source_roots.clone();
        records.push(record);
        records.sort_by_key(manual_source_id);
        save_manual_source_roots(&self.config.data_root, &records)?;
        self.manual_source_roots = records;
        self.config.sources.push(approved);
        self.source_catalogs.clear();
        self.plans.clear();
        Ok(id)
    }

    pub fn remove_manual_source_folder(&mut self, id: &str) -> Result<(), AppError> {
        if !id.starts_with("source:manual:")
            || !self
                .manual_source_roots
                .iter()
                .any(|record| manual_source_id(record) == id)
        {
            return Err(AppError::UnsafePath(PathBuf::from(id)));
        }
        let records: Vec<_> = self
            .manual_source_roots
            .iter()
            .filter(|record| manual_source_id(record) != id)
            .cloned()
            .collect();
        save_manual_source_roots(&self.config.data_root, &records)?;
        self.manual_source_roots = records;
        self.config.sources.retain(|root| root.id != id);
        self.source_catalogs.clear();
        self.plans.clear();
        Ok(())
    }

    fn manual_source_summaries(&self) -> Vec<ManualSourceRootSummary> {
        self.manual_source_roots
            .iter()
            .map(|record| {
                let validation = approved_manual_source(record);
                let id = validation
                    .as_ref()
                    .map_or_else(|_| manual_source_id(record), |(_, root)| root.id.clone());
                let active = self.config.sources.iter().any(|root| root.id == id);
                ManualSourceRootSummary {
                    id,
                    path: record.path.clone(),
                    source_app: record.source_app,
                    source_kind: record.source_kind,
                    active,
                    error: validation.err().map(|error| error.to_string()).or_else(|| {
                        (!active).then(|| {
                            "folder duplicates an automatically discovered source".to_owned()
                        })
                    }),
                }
            })
            .collect()
    }

    pub fn discover(&self) -> DiscoveryResponse {
        DiscoveryResponse {
            source_root_ids: self
                .config
                .sources
                .iter()
                .map(|item| item.id.clone())
                .collect(),
            manual_source_roots: self.manual_source_summaries(),
            target_catalog_ids: self
                .config
                .targets
                .iter()
                .map(|item| item.id.clone())
                .collect(),
            accounts: self
                .config
                .accounts
                .iter()
                .map(|item| item.account.clone())
                .collect(),
            orca_slicer_detected: self
                .config
                .sources
                .iter()
                .any(|item| item.source_app == SourceApp::OrcaSlicer),
            bambu_studio_detected: !self.config.targets.is_empty()
                || self
                    .config
                    .sources
                    .iter()
                    .any(|item| item.source_app == SourceApp::BambuStudio),
            platform: std::env::consts::OS.to_owned(),
        }
    }

    pub fn catalog_sources(
        &mut self,
        request: CatalogSourcesRequest,
    ) -> Result<SourceCatalogResponse, AppError> {
        self.catalog_sources_with_progress(request, |_| {})
    }

    pub fn catalog_sources_with_progress(
        &mut self,
        request: CatalogSourcesRequest,
        mut on_progress: impl FnMut(CatalogProgressEvent),
    ) -> Result<SourceCatalogResponse, AppError> {
        if request.root_ids.is_empty() {
            return Err(AppError::InvalidProfile(
                "no source roots selected".to_owned(),
            ));
        }
        let roots: Vec<_> = request
            .root_ids
            .iter()
            .map(|id| {
                self.config
                    .sources
                    .iter()
                    .find(|root| &root.id == id)
                    .cloned()
                    .ok_or_else(|| {
                        AppError::UnsafePath(PathBuf::from(format!(
                            "unapproved source root id: {id}"
                        )))
                    })
            })
            .collect::<Result<_, _>>()?;
        let loading_total = profile_file_count(&roots);
        let mut loaded = 0;
        on_progress(CatalogProgressEvent {
            phase: CatalogProgressPhase::Loading,
            processed: 0,
            total: loading_total,
        });
        let mut roots_by_app = BTreeMap::<SourceApp, Vec<ApprovedSourceRoot>>::new();
        for root in &roots {
            roots_by_app
                .entry(root.source_app)
                .or_default()
                .push(root.clone());
        }
        let mut catalogs = BTreeMap::new();
        for (source_app, app_roots) in roots_by_app {
            let catalog = load_source_catalog_with_progress(&app_roots, || {
                loaded += 1;
                if loaded == loading_total || loaded % CATALOG_PROGRESS_BATCH_SIZE == 0 {
                    on_progress(CatalogProgressEvent {
                        phase: CatalogProgressPhase::Loading,
                        processed: loaded,
                        total: loading_total,
                    });
                }
            })?;
            catalogs.insert(source_app, catalog);
        }
        if loaded != loading_total {
            on_progress(CatalogProgressEvent {
                phase: CatalogProgressPhase::Loading,
                processed: loaded,
                total: loading_total,
            });
        }
        let resolving_total = catalogs
            .values()
            .map(|catalog| catalog.selectable_profiles().len())
            .sum();
        let mut resolved = 0;
        on_progress(CatalogProgressEvent {
            phase: CatalogProgressPhase::Resolving,
            processed: 0,
            total: resolving_total,
        });
        let mut profile_locators = BTreeMap::new();
        let mut sources = BTreeMap::new();
        for (source_app, catalog) in &catalogs {
            for record in catalog.selectable_profiles() {
                let effective = catalog.resolve_name(&record.name)?;
                let display = display_name(&record.name);
                let vendor = first_string(&effective.values, "filament_vendor");
                let material = first_string(&effective.values, "filament_type");
                let (family, variant) = family_variant(&display, &vendor, &material);
                let compatible_printers = effective
                    .string_values("compatible_printers")
                    .into_iter()
                    .map(str::to_owned)
                    .collect();
                let existing_filament_id = effective
                    .value("filament_id")
                    .and_then(Value::as_str)
                    .filter(|id| id.starts_with('P'))
                    .map(str::to_owned);
                let id = source_profile_id(*source_app, &record.name);
                profile_locators.insert(
                    id.clone(),
                    SourceProfileLocator {
                        source_app: *source_app,
                        profile_name: record.name.clone(),
                    },
                );
                sources.insert(
                    id.0.clone(),
                    MigrationSource {
                        id,
                        name: display,
                        vendor,
                        material,
                        family,
                        variant,
                        source_app: record.source_app,
                        source_kind: record.source_kind,
                        compatible_printers,
                        migration_status: MigrationStatus::New,
                        source_precondition_fingerprint: effective_settings_fingerprint(
                            &effective,
                        )?,
                        existing_filament_id,
                    },
                );
                resolved += 1;
                if resolved == resolving_total || resolved % CATALOG_PROGRESS_BATCH_SIZE == 0 {
                    on_progress(CatalogProgressEvent {
                        phase: CatalogProgressPhase::Resolving,
                        processed: resolved,
                        total: resolving_total,
                    });
                }
            }
        }
        on_progress(CatalogProgressEvent {
            phase: CatalogProgressPhase::Finished,
            processed: resolving_total,
            total: resolving_total,
        });
        let response_sources = sources.values().map(CatalogSource::from).collect();
        let catalog_id = format!("sources:{}", Uuid::new_v4().simple());
        self.source_catalogs.insert(
            catalog_id.clone(),
            CachedSources {
                roots,
                sources,
                catalogs,
                profile_locators,
            },
        );
        self.plans.clear();
        Ok(SourceCatalogResponse {
            catalog_id,
            sources: response_sources,
        })
    }

    pub fn catalog_targets(
        &mut self,
        request: CatalogTargetsRequest,
    ) -> Result<TargetCatalogResponse, AppError> {
        self.catalog_targets_with_progress(request, |_| {})
    }

    pub fn catalog_targets_with_progress(
        &mut self,
        request: CatalogTargetsRequest,
        mut on_progress: impl FnMut(CatalogProgressEvent),
    ) -> Result<TargetCatalogResponse, AppError> {
        let approved = self
            .config
            .targets
            .iter()
            .find(|target| target.id == request.catalog_id)
            .cloned()
            .ok_or_else(|| {
                AppError::UnsafePath(PathBuf::from(format!(
                    "unapproved target catalog id: {}",
                    request.catalog_id
                )))
            })?;
        let loading_total = json_file_count(&approved.profile_root);
        let mut loaded = 0;
        on_progress(CatalogProgressEvent {
            phase: CatalogProgressPhase::Loading,
            processed: 0,
            total: loading_total,
        });
        let catalog = TargetCatalog::load_with_progress(
            &approved.manifest_path,
            approved.custom_machine_root.as_deref(),
            || {
                loaded += 1;
                if loaded == loading_total || loaded % CATALOG_PROGRESS_BATCH_SIZE == 0 {
                    on_progress(CatalogProgressEvent {
                        phase: CatalogProgressPhase::Loading,
                        processed: loaded,
                        total: loading_total,
                    });
                }
            },
        )?;
        if loaded != loading_total {
            on_progress(CatalogProgressEvent {
                phase: CatalogProgressPhase::Loading,
                processed: loaded,
                total: loading_total,
            });
        }
        let resolving_total = loading_total;
        let mut resolved = 0;
        on_progress(CatalogProgressEvent {
            phase: CatalogProgressPhase::Resolving,
            processed: 0,
            total: resolving_total,
        });
        let profiles = ProfileCatalog::load_roots_with_progress(
            &[CatalogRoot::system(
                approved.profile_root.clone(),
                SourceApp::BambuStudio,
            )],
            || {
                resolved += 1;
                if resolved == resolving_total || resolved % CATALOG_PROGRESS_BATCH_SIZE == 0 {
                    on_progress(CatalogProgressEvent {
                        phase: CatalogProgressPhase::Resolving,
                        processed: resolved,
                        total: resolving_total,
                    });
                }
            },
        )?;
        if resolved != resolving_total {
            on_progress(CatalogProgressEvent {
                phase: CatalogProgressPhase::Resolving,
                processed: resolved,
                total: resolving_total,
            });
        }
        on_progress(CatalogProgressEvent {
            phase: CatalogProgressPhase::Finished,
            processed: resolving_total,
            total: resolving_total,
        });
        let printers = catalog
            .visible(request.show_custom)
            .into_iter()
            .cloned()
            .collect();
        let catalog_id = format!("targets:{}", Uuid::new_v4().simple());
        self.target_catalogs.insert(
            catalog_id.clone(),
            CachedTargets {
                approved,
                catalog,
                profiles,
            },
        );
        self.plans.clear();
        Ok(TargetCatalogResponse {
            catalog_id,
            printers,
        })
    }

    pub fn printer_artwork(&self, catalog_id: &str, printer_id: &str) -> Result<Vec<u8>, AppError> {
        let target_cache = self
            .target_catalogs
            .get(catalog_id)
            .ok_or_else(|| AppError::Conflict("target catalog is stale".to_owned()))?;
        if !printer_id.starts_with("official:") {
            return Err(AppError::InvalidProfile(
                "printer artwork is available only for official targets".to_owned(),
            ));
        }
        let path = target_cache
            .catalog
            .artwork_path(printer_id)
            .ok_or_else(|| {
                AppError::InvalidProfile(format!("printer artwork is unavailable: {printer_id}"))
            })?;
        let canonical = path
            .canonicalize()
            .map_err(|error| AppError::io(path, error))?;
        if canonical != path || !canonical.is_file() {
            return Err(AppError::UnsafePath(canonical));
        }
        std::fs::read(&canonical).map_err(|error| AppError::io(&canonical, error))
    }

    pub fn build_plan(&mut self, request: BuildPlanRequest) -> Result<BuildPlanResponse, AppError> {
        self.build_plan_with_decisions(request, &[])
    }

    pub fn resolve_plan_dependencies(
        &mut self,
        request: ResolvePlanDependenciesRequest,
    ) -> Result<BuildPlanResponse, AppError> {
        self.build_plan_with_decisions(request.request, &request.decisions)
    }

    fn build_plan_with_decisions(
        &mut self,
        request: BuildPlanRequest,
        decisions: &[TargetTemplateDecision],
    ) -> Result<BuildPlanResponse, AppError> {
        let outputs = request.outputs;
        if !outputs.slicing_presets && !outputs.custom_filaments {
            return Err(AppError::InvalidProfile(
                "at least one migration output must be enabled".to_owned(),
            ));
        }
        let mut decision_map = BTreeMap::new();
        for decision in decisions {
            if decision_map
                .insert(decision.issue_id.clone(), decision)
                .is_some()
            {
                return Err(AppError::InvalidProfile(format!(
                    "duplicate target template decision: {}",
                    decision.issue_id
                )));
            }
        }
        let mut used_decisions = BTreeSet::new();
        let account = self.approved_eligible_account(&request.destination_account_id)?;
        let source_cache = self
            .source_catalogs
            .get(&request.source_catalog_id)
            .ok_or_else(|| AppError::Conflict("source catalog is stale".to_owned()))?;
        let target_cache = self
            .target_catalogs
            .get(&request.target_catalog_id)
            .ok_or_else(|| AppError::Conflict("target catalog is stale".to_owned()))?;
        let sources: Vec<_> = request
            .source_ids
            .iter()
            .map(|id| {
                source_cache.sources.get(id).cloned().ok_or_else(|| {
                    AppError::InvalidProfile(format!("source is not selectable: {id}"))
                })
            })
            .collect::<Result<_, _>>()?;
        let mut targets = Vec::new();
        for selection in &request.nozzles {
            let printer = target_cache
                .catalog
                .printers()
                .iter()
                .find(|printer| printer.id == selection.printer_id)
                .ok_or_else(|| {
                    AppError::InvalidProfile(format!(
                        "target printer is not available: {}",
                        selection.printer_id
                    ))
                })?;
            for diameter in &selection.diameters {
                let nozzle = printer
                    .nozzles
                    .iter()
                    .find(|nozzle| &nozzle.diameter == diameter)
                    .ok_or_else(|| {
                        AppError::InvalidProfile(format!(
                            "target nozzle is not available: {} {diameter}",
                            printer.name
                        ))
                    })?;
                if !nozzle.supported {
                    return Err(AppError::InvalidProfile(format!(
                        "target nozzle is unsupported: {} {diameter}",
                        printer.name
                    )));
                }
                targets.push(TargetSelection {
                    printer_id: printer.id.clone(),
                    printer_name: printer.name.clone(),
                    printer_code: printer.code.clone(),
                    nozzle: nozzle.diameter.clone(),
                    printer_preset_name: nozzle.printer_preset_name.clone(),
                    custom_unverified: !printer.verified,
                });
            }
        }
        let migration_request = MigrationRequest {
            sources,
            targets,
            preset_template: request.preset_template,
            ams_template: request.ams_template,
            user_id: account.account.id.clone(),
        };
        let adapter = BambuAdapter::v2_0_0_56();
        let mut material_fingerprints = MaterialFingerprintIndex::new();
        let mut target_profile_names = BTreeMap::new();
        let mut dependency_issues = BTreeMap::<String, TargetTemplateIssue>::new();
        for source in &migration_request.sources {
            let locator = source_cache
                .profile_locators
                .get(&source.id)
                .ok_or_else(|| {
                    AppError::InvalidProfile(format!("source locator is missing: {}", source.id))
                })?;
            let source_effective = source_cache
                .catalogs
                .get(&locator.source_app)
                .ok_or_else(|| {
                    AppError::InvalidProfile(format!("source catalog is missing: {}", source.id))
                })?
                .resolve_name(&locator.profile_name)?;
            for target in &migration_request.targets {
                let expected = format!("Generic {} @BBL {}", source.material, target.printer_code);
                let exact = target_cache
                    .profiles
                    .resolve_name(&expected)
                    .ok()
                    .filter(|profile| target_profile_matches_material(profile, &source.material));
                let (target_effective, target_profile_reference) = if let Some(profile) = exact {
                    (profile, TargetProfileReference::Catalog(expected.clone()))
                } else {
                    let issue_key = format!(
                        "{}\u{1f}{}\u{1f}{}",
                        source.material.to_ascii_lowercase(),
                        target.printer_id,
                        target.nozzle
                    );
                    let issue_id = sha256_bytes(issue_key.as_bytes());
                    let candidates = installed_target_candidates(
                        &target_cache.profiles,
                        &source.material,
                        target,
                    );
                    let source_candidates =
                        source_target_candidates(source_cache, &source.material, target, &adapter);
                    if let Some(decision) = decision_map.get(&issue_id) {
                        used_decisions.insert(issue_id.clone());
                        match decision.action {
                            TargetTemplateDecisionAction::UseInstalled => {
                                if decision.source_id.is_some() {
                                    return Err(AppError::InvalidProfile(format!(
                                        "installed target decision cannot include a source id: {issue_id}"
                                    )));
                                }
                                let profile_name = decision.profile_name.as_ref().ok_or_else(|| {
                                    AppError::InvalidProfile(format!(
                                        "installed target decision is missing a profile: {issue_id}"
                                    ))
                                })?;
                                if !candidates.contains(profile_name) {
                                    return Err(AppError::InvalidProfile(format!(
                                        "target template decision is not an approved candidate: {profile_name}"
                                    )));
                                }
                                let profile = target_cache.profiles.resolve_name(profile_name)?;
                                if !target_profile_supports(&profile, &source.material, target) {
                                    return Err(AppError::InvalidProfile(format!(
                                        "target template decision is incompatible: {profile_name}"
                                    )));
                                }
                                (
                                    profile,
                                    TargetProfileReference::Catalog(profile_name.clone()),
                                )
                            }
                            TargetTemplateDecisionAction::UseSource => {
                                if decision.profile_name.is_some() {
                                    return Err(AppError::InvalidProfile(format!(
                                        "source target decision cannot include a profile name: {issue_id}"
                                    )));
                                }
                                let source_id = decision.source_id.as_ref().ok_or_else(|| {
                                    AppError::InvalidProfile(format!(
                                        "source target decision is missing a source id: {issue_id}"
                                    ))
                                })?;
                                if !source_candidates
                                    .iter()
                                    .any(|candidate| candidate.source_id == *source_id)
                                {
                                    return Err(AppError::InvalidProfile(format!(
                                        "target source decision is not an approved candidate: {source_id}"
                                    )));
                                }
                                if !request.source_ids.contains(source_id) {
                                    return Err(AppError::InvalidProfile(format!(
                                        "target source must be included in the migration: {source_id}"
                                    )));
                                }
                                let target_source =
                                    source_cache.sources.get(source_id).ok_or_else(|| {
                                        AppError::InvalidProfile(format!(
                                            "target source is no longer selectable: {source_id}"
                                        ))
                                    })?;
                                let locator = source_cache
                                    .profile_locators
                                    .get(&target_source.id)
                                    .ok_or_else(|| {
                                        AppError::InvalidProfile(format!(
                                            "target source locator is missing: {source_id}"
                                        ))
                                    })?;
                                let profile = source_cache
                                    .catalogs
                                    .get(&locator.source_app)
                                    .ok_or_else(|| {
                                        AppError::InvalidProfile(format!(
                                            "target source catalog is missing: {source_id}"
                                        ))
                                    })?
                                    .resolve_name(&locator.profile_name)?;
                                if !target_profile_supports(&profile, &source.material, target) {
                                    return Err(AppError::InvalidProfile(format!(
                                        "target source decision is incompatible: {source_id}"
                                    )));
                                }
                                (
                                    profile,
                                    TargetProfileReference::Source(target_source.id.clone()),
                                )
                            }
                        }
                    } else {
                        let issue =
                            dependency_issues
                                .entry(issue_key.clone())
                                .or_insert_with(|| TargetTemplateIssue {
                                    id: issue_id,
                                    expected_name: expected.clone(),
                                    material: source.material.clone(),
                                    printer_id: target.printer_id.clone(),
                                    printer_name: target.printer_name.clone(),
                                    nozzle: target.nozzle.clone(),
                                    affected_sources: Vec::new(),
                                    installed_candidates: candidates
                                        .into_iter()
                                        .enumerate()
                                        .map(|(index, profile_name)| {
                                            TargetTemplateInstalledCandidate {
                                                profile_name,
                                                recommended: index == 0,
                                            }
                                        })
                                        .collect(),
                                    source_candidates: source_candidates.clone(),
                                    diagnostic: format!(
                                        "{expected} is not installed for {} {} mm",
                                        target.printer_name, target.nozzle
                                    ),
                                });
                        if !issue
                            .affected_sources
                            .iter()
                            .any(|affected| affected.id == source.id.0)
                        {
                            issue.affected_sources.push(TargetTemplateAffectedSource {
                                id: source.id.0.clone(),
                                name: source.name.clone(),
                            });
                        }
                        continue;
                    }
                };
                let key = (source.id.clone(), target.printer_preset_name.clone());
                material_fingerprints.insert(
                    key.clone(),
                    generated_material_settings_fingerprint(
                        &source_effective,
                        &target_effective,
                        &adapter,
                    )?,
                );
                target_profile_names.insert(key, target_profile_reference);
            }
        }
        if used_decisions.len() != decision_map.len() {
            return Err(AppError::InvalidProfile(
                "target template decisions are stale or unrelated to this plan".to_owned(),
            ));
        }
        if !dependency_issues.is_empty() {
            return Ok(BuildPlanResponse::NeedsResolution {
                issues: dependency_issues.into_values().collect(),
            });
        }
        let destinations = if outputs.custom_filaments {
            destination_index(&account.account.path, &adapter)?
        } else {
            DestinationIndex::default()
        };
        let mut plan = Planner::build_with_context(
            &migration_request,
            &destinations,
            &request.naming,
            &material_fingerprints,
        )?;
        reconcile_normal_destinations(
            &mut plan,
            outputs,
            &account.account.path,
            &adapter,
            &request.naming,
        )?;
        let canonical = serde_json::to_vec(&(&plan.operations, outputs))
            .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
        plan.id = sha256_bytes(&canonical);
        let mut target_profiles = BTreeMap::new();
        for operation in &plan.operations {
            let key = (
                operation.source_id.clone(),
                operation.printer_preset_name.clone(),
            );
            let candidate = target_profile_names.get(&key).ok_or_else(|| {
                AppError::InvalidProfile("planned target template disappeared".to_owned())
            })?;
            target_profiles.insert(operation.id.clone(), candidate.clone());
        }
        let destination_preconditions =
            snapshot_destination(&plan, outputs, &account.account.path)?;
        self.plans.insert(
            plan.id.clone(),
            StoredPlan {
                plan: plan.clone(),
                source_roots: source_cache.roots.clone(),
                source_profile_locators: source_cache.profile_locators.clone(),
                target: target_cache.approved.clone(),
                target_profiles,
                destination_preconditions,
                outputs,
                destination_account_id: account.account.id.clone(),
            },
        );
        Ok(BuildPlanResponse::Ready { plan })
    }

    pub fn execute_plan_with(
        &mut self,
        request: ExecutePlanRequest,
        process: &mut impl ProcessBackend,
        clock: &mut impl Clock,
    ) -> Result<LocalRunResult, AppError> {
        self.execute_plan_with_progress(request, process, clock, |_| {})
    }

    pub fn execute_plan_with_progress(
        &mut self,
        request: ExecutePlanRequest,
        process: &mut impl ProcessBackend,
        clock: &mut impl Clock,
        mut on_progress: impl FnMut(ExecutionPhase),
    ) -> Result<LocalRunResult, AppError> {
        on_progress(ExecutionPhase::Validating);
        let stored = self
            .plans
            .get(&request.plan_id)
            .cloned()
            .ok_or_else(|| AppError::Conflict("plan id is stale or unknown".to_owned()))?;
        if stored
            .plan
            .operations
            .iter()
            .any(|operation| operation.action == PlanAction::Block)
        {
            return Err(AppError::Conflict(
                "plan contains unresolved blocking conflicts".to_owned(),
            ));
        }
        let has_writes = stored
            .plan
            .operations
            .iter()
            .any(|operation| operation.action != PlanAction::Skip);
        let account = self.approved_eligible_account(&stored.destination_account_id)?;
        if has_writes {
            on_progress(ExecutionPhase::ClosingBambu);
            ProcessController::ensure_stopped(
                process,
                clock,
                self.config.process_close_timeout_ms,
                50,
            )?;
        }
        on_progress(ExecutionPhase::Staging);
        let sources = resolve_stored_sources(
            &stored.source_roots,
            &stored.source_profile_locators,
            &stored.plan,
        )?;
        let targets = ProfileCatalog::load_roots(&[CatalogRoot::system(
            stored.target.profile_root.clone(),
            SourceApp::BambuStudio,
        )])?;
        let run_id = Uuid::new_v4().simple().to_string();
        let staging_root = self.config.data_root.join("staging").join(&run_id);
        let updated_time = chrono::Utc::now().timestamp();
        let existing_sidecars = if stored.outputs.custom_filaments {
            load_existing_sidecars(&stored.plan, &account.account.path)?
        } else {
            BTreeMap::new()
        };
        let existing_normal_compatibility = if stored.outputs.slicing_presets {
            load_existing_normal_compatibility(&stored.plan, &account.account.path)?
        } else {
            BTreeMap::new()
        };
        let mut staged = Writer::stage(
            &stored.plan,
            &WriterContext {
                sources,
                targets: &targets,
                target_profiles: stored.target_profiles.clone(),
                adapter: BambuAdapter::v2_0_0_56(),
                outputs: stored.outputs,
                updated_time,
                existing_sidecars,
                existing_normal_compatibility,
            },
            &staging_root,
        )?;
        on_progress(ExecutionPhase::ValidatingOutput);
        let result = (|| {
            for artifact in &mut staged.artifacts {
                artifact.destination_precondition = Some(
                    stored
                        .destination_preconditions
                        .get(&artifact.relative_path)
                        .cloned()
                        .ok_or_else(|| {
                            AppError::Conflict(format!(
                                "planned destination snapshot is missing: {}",
                                artifact.relative_path.display()
                            ))
                        })?,
                );
            }
            let run_root = self.config.data_root.join("runs").join(&run_id);
            on_progress(ExecutionPhase::BackingUp);
            let mut transaction = Transaction::preflight(
                &stored.plan,
                &staged,
                &account.account.path,
                &run_root,
                TransactionOptions {
                    run_id: run_id.clone(),
                    created_at: updated_time,
                },
            )?;
            on_progress(ExecutionPhase::Committing);
            let outcome = transaction.commit()?;
            on_progress(ExecutionPhase::WritingReceipt);
            let receipt = RunReceipt::from_transaction(&transaction, ReceiptState::Committed)?;
            let receipt_path = run_root.join("receipt.json");
            receipt.save(&receipt_path)?;
            self.runs.insert(run_id.clone(), run_root);
            on_progress(ExecutionPhase::Finished);
            Ok(LocalRunResult {
                run_id,
                plan_id: stored.plan.id,
                committed_files: outcome.committed,
                created_files: receipt.counts.creates,
                updated_files: receipt.counts.updates,
                deleted_files: receipt.counts.deletes,
                skipped_operations: stored
                    .plan
                    .operations
                    .iter()
                    .filter(|operation| operation.action == PlanAction::Skip)
                    .count(),
                backup_sha256: receipt.backup.sha256.clone(),
                backup_file_count: receipt.backup.file_count,
                receipt_path,
            })
        })();
        let _ = std::fs::remove_dir_all(staging_root);
        result
    }

    pub fn synchronize_run_with(
        &self,
        run_id: &str,
        process: &mut impl ProcessBackend,
        runtime: &mut impl SyncRuntime,
        timeout_ms: u64,
        poll_interval_ms: u64,
    ) -> Result<SyncResult, AppError> {
        self.synchronize_run_with_progress(
            run_id,
            process,
            runtime,
            timeout_ms,
            poll_interval_ms,
            |_| {},
        )
    }

    pub fn synchronize_run_with_progress(
        &self,
        run_id: &str,
        process: &mut impl ProcessBackend,
        runtime: &mut impl SyncRuntime,
        timeout_ms: u64,
        poll_interval_ms: u64,
        mut on_progress: impl FnMut(SyncPhase),
    ) -> Result<SyncResult, AppError> {
        let run_root = self.known_run_root(run_id)?;
        let transaction = Transaction::load(&run_root)?;
        let account = self
            .config
            .accounts
            .iter()
            .find(|account| account.account.path == transaction.journal().destination_root)
            .ok_or_else(|| {
                AppError::Conflict("run destination is no longer approved".to_owned())
            })?;
        let executable = account.bambu_executable.as_deref().ok_or_else(|| {
            AppError::Conflict("Bambu Studio executable is unavailable".to_owned())
        })?;
        let setting_id_prefix = BambuAdapter::v2_0_0_56().setting_id_prefix.to_owned();
        let mut expectations = Vec::new();
        for entry in &transaction.journal().entries {
            if !entry.committed
                || entry
                    .relative_path
                    .extension()
                    .and_then(|value| value.to_str())
                    != Some("info")
            {
                continue;
            }
            let committed_sha256 = entry.committed_sha256.as_ref().ok_or_else(|| {
                AppError::Conflict(format!(
                    "committed sidecar has no journal hash: {}",
                    entry.destination_path.display()
                ))
            })?;
            let current = std::fs::read(&entry.destination_path)
                .map_err(|error| AppError::io(&entry.destination_path, error))?;
            if sha256_bytes(&current) != *committed_sha256 {
                return Err(AppError::Conflict(format!(
                    "sidecar changed before synchronization: {}",
                    entry.destination_path.display()
                )));
            }
            expectations.extend(
                entry
                    .operation_ids
                    .iter()
                    .map(|operation_id| SyncExpectation {
                        operation_id: operation_id.clone(),
                        info_path: entry.destination_path.clone(),
                        initial_sha256: committed_sha256.clone(),
                        setting_id_prefix: setting_id_prefix.clone(),
                    }),
            );
        }
        if expectations.is_empty() {
            return Err(AppError::Conflict(
                "run has no committed sidecars to synchronize".to_owned(),
            ));
        }
        on_progress(SyncPhase::Launching);
        ProcessController::launch(process, executable)?;
        on_progress(SyncPhase::Monitoring);
        let result = SyncMonitor::wait(&expectations, runtime, timeout_ms, poll_interval_ms)?;
        let receipt_path = run_root.join("receipt.json");
        let mut receipt = RunReceipt::load(&receipt_path)?;
        receipt.synchronization = Some(result.clone());
        receipt.save(&receipt_path)?;
        on_progress(SyncPhase::Finished);
        Ok(result)
    }

    pub fn restore_preview(&self, run_id: &str) -> Result<RestorePreview, AppError> {
        Transaction::load(&self.known_run_root(run_id)?)?.restore_preview()
    }

    pub fn restore_owned(&mut self, run_id: &str) -> Result<RollbackOutcome, AppError> {
        let run_root = self.known_run_root(run_id)?;
        let receipt_path = run_root.join("receipt.json");
        let previous_receipt = RunReceipt::load(&receipt_path)?;
        let mut transaction = Transaction::load(&run_root)?;
        let outcome = transaction.rollback_owned()?;
        let state = if outcome.external_conflicts == 0 {
            ReceiptState::RolledBack
        } else {
            ReceiptState::PartialRollback
        };
        let mut receipt = RunReceipt::from_transaction(&transaction, state)?;
        receipt.synchronization = previous_receipt.synchronization;
        receipt.ams_verification = previous_receipt.ams_verification;
        receipt.save(&receipt_path)?;
        Ok(outcome)
    }

    pub fn record_ams_verification(&self, request: AmsVerificationRequest) -> Result<(), AppError> {
        let run_root = self.known_run_root(&request.run_id)?;
        let transaction = Transaction::load(&run_root)?;
        let known: BTreeSet<_> = transaction
            .journal()
            .entries
            .iter()
            .flat_map(|entry| entry.operation_ids.iter().cloned())
            .collect();
        if request.operation_ids.is_empty()
            || request
                .operation_ids
                .iter()
                .any(|operation| !known.contains(operation))
        {
            return Err(AppError::InvalidProfile(
                "AMS verification contains an unknown operation".to_owned(),
            ));
        }
        let mut receipt = RunReceipt::load(&run_root.join("receipt.json"))?;
        let synchronization = receipt.synchronization.as_ref().ok_or_else(|| {
            AppError::Conflict("AMS verification requires cloud assignment evidence".to_owned())
        })?;
        if request.operation_ids.iter().any(|operation_id| {
            !synchronization.observations.iter().any(|observation| {
                observation.operation_id == *operation_id
                    && observation.evidence == EvidenceLevel::CloudIdAssigned
            })
        }) {
            return Err(AppError::Conflict(
                "AMS verification requires cloud assignment evidence for every operation"
                    .to_owned(),
            ));
        }
        let recorded_at = chrono::Utc::now().timestamp();
        let evidence = AmsVerificationEvidence {
            operation_ids: request.operation_ids.clone(),
            recorded_at,
        };
        let record = AmsVerificationRecord {
            run_id: request.run_id,
            operation_ids: request.operation_ids,
            note: request.note,
            recorded_at,
            operator_verified: true,
        };
        let path = run_root.join("ams-verification.json");
        let mut bytes = serde_json::to_vec_pretty(&record)
            .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes).map_err(|error| AppError::io(path, error))?;
        receipt.ams_verification = Some(evidence);
        receipt.save(&run_root.join("receipt.json"))
    }

    pub fn preview_names(
        &self,
        request: PreviewNamesRequest,
    ) -> Result<Vec<PreviewNameResult>, AppError> {
        let preset = NamingTemplate::parse(request.preset_template)?;
        let ams = NamingTemplate::parse(request.ams_template)?;
        let preset_rules = request
            .preset_rules
            .iter()
            .map(ReplacementRuleSpec::compile)
            .collect::<Result<Vec<_>, _>>()?;
        let ams_rules = request
            .ams_rules
            .iter()
            .map(ReplacementRuleSpec::compile)
            .collect::<Result<Vec<_>, _>>()?;
        request
            .rows
            .into_iter()
            .map(|row| {
                let context = NamingContext::new(&row.source_name, &row.vendor, &row.material)
                    .with("family", row.family)
                    .with("variant", row.variant)
                    .with("source_app", source_app_label(row.source_app))
                    .with("source_kind", source_kind_label(row.source_kind))
                    .with("printer", row.printer)
                    .with("printer_code", row.printer_code)
                    .with("nozzle", row.nozzle);
                let preset_before = preset.render(&context)?;
                let ams_before = ams.render(&context)?;
                Ok(PreviewNameResult {
                    preset_name: NamingTemplate::apply_rules_with_context(
                        &preset_before,
                        &preset_rules,
                        &context,
                    )?,
                    ams_name: NamingTemplate::apply_rules_with_context(
                        &ams_before,
                        &ams_rules,
                        &context,
                    )?,
                    preset_before,
                    ams_before,
                })
            })
            .collect()
    }

    pub fn data_root(&self) -> &Path {
        &self.config.data_root
    }

    fn approved_eligible_account(&self, id: &str) -> Result<ApprovedAccount, AppError> {
        let approved = self
            .config
            .accounts
            .iter()
            .find(|account| account.account.id == id)
            .cloned()
            .ok_or_else(|| {
                AppError::UnsafePath(PathBuf::from(format!("unapproved account id: {id}")))
            })?;
        let current = inspect_account(&approved.account.path)?;
        if current.eligibility != AccountEligibility::Eligible {
            return Err(AppError::Conflict(format!(
                "destination account is not eligible: {:?}",
                current.eligibility
            )));
        }
        Ok(ApprovedAccount {
            account: current,
            bambu_executable: approved.bambu_executable,
        })
    }

    fn known_run_root(&self, run_id: &str) -> Result<PathBuf, AppError> {
        validate_opaque_id(run_id)?;
        let path = self
            .runs
            .get(run_id)
            .cloned()
            .unwrap_or_else(|| self.config.data_root.join("runs").join(run_id));
        let canonical = path
            .canonicalize()
            .map_err(|error| AppError::io(&path, error))?;
        let runs_root = self.config.data_root.join("runs");
        let runs_root = runs_root
            .canonicalize()
            .map_err(|error| AppError::io(&runs_root, error))?;
        if canonical.starts_with(runs_root) {
            Ok(canonical)
        } else {
            Err(AppError::UnsafePath(canonical))
        }
    }
}

impl From<&MigrationSource> for CatalogSource {
    fn from(source: &MigrationSource) -> Self {
        Self {
            id: source.id.clone(),
            name: source.name.clone(),
            vendor: source.vendor.clone(),
            material: source.material.clone(),
            family: source.family.clone(),
            variant: source.variant.clone(),
            source_app: source.source_app,
            source_kind: source.source_kind,
            compatible_printers: source.compatible_printers.clone(),
            migration_status: source.migration_status,
            warnings: Vec::new(),
        }
    }
}

fn canonicalize_config(config: &mut ServiceConfig) -> Result<(), AppError> {
    let mut ids = BTreeSet::new();
    for root in &mut config.sources {
        if !ids.insert(root.id.clone()) {
            return Err(AppError::InvalidProfile(format!(
                "duplicate approved id: {}",
                root.id
            )));
        }
        root.path = root
            .path
            .canonicalize()
            .map_err(|error| AppError::io(&root.path, error))?;
    }
    for target in &mut config.targets {
        if !ids.insert(target.id.clone()) {
            return Err(AppError::InvalidProfile(format!(
                "duplicate approved id: {}",
                target.id
            )));
        }
        target.manifest_path = target
            .manifest_path
            .canonicalize()
            .map_err(|error| AppError::io(&target.manifest_path, error))?;
        target.profile_root = target
            .profile_root
            .canonicalize()
            .map_err(|error| AppError::io(&target.profile_root, error))?;
        if let Some(path) = &mut target.custom_machine_root {
            *path = path
                .canonicalize()
                .map_err(|error| AppError::io(&*path, error))?;
        }
    }
    for account in &mut config.accounts {
        if !ids.insert(format!("account:{}", account.account.id)) {
            return Err(AppError::InvalidProfile(format!(
                "duplicate approved account: {}",
                account.account.id
            )));
        }
        account.account.path = account
            .account
            .path
            .canonicalize()
            .map_err(|error| AppError::io(&account.account.path, error))?;
        if let Some(executable) = &mut account.bambu_executable {
            *executable = executable
                .canonicalize()
                .map_err(|error| AppError::io(&*executable, error))?;
        }
    }
    if config.data_root.exists() {
        config.data_root = config
            .data_root
            .canonicalize()
            .map_err(|error| AppError::io(&config.data_root, error))?;
    }
    Ok(())
}

fn json_file_count(root: &Path) -> usize {
    walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
                && entry.path().file_name().and_then(|value| value.to_str()) != Some("BBL.json")
        })
        .count()
}

fn profile_file_count(roots: &[ApprovedSourceRoot]) -> usize {
    roots.iter().map(|root| json_file_count(&root.path)).sum()
}

fn load_source_catalog(roots: &[ApprovedSourceRoot]) -> Result<ProfileCatalog, AppError> {
    load_source_catalog_with_progress(roots, || {})
}

fn load_source_catalog_with_progress(
    roots: &[ApprovedSourceRoot],
    on_file: impl FnMut(),
) -> Result<ProfileCatalog, AppError> {
    let roots: Vec<_> = roots
        .iter()
        .map(|root| match root.source_kind {
            SourceKind::FactorySystem if is_orca_filament_library_root(&root.path) => {
                CatalogRoot::filament_library(root.path.clone(), root.source_app)
            }
            SourceKind::FactorySystem => {
                CatalogRoot::vendor_system(root.path.clone(), root.source_app)
            }
            SourceKind::UserCustom if is_default_local_profile_root(&root.path) => {
                CatalogRoot::default_local_user(root.path.clone(), root.source_app)
            }
            SourceKind::UserCustom => CatalogRoot::user(root.path.clone(), root.source_app),
        })
        .collect();
    ProfileCatalog::load_roots_with_progress(&roots, on_file)
}

fn is_orca_filament_library_root(path: &Path) -> bool {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("OrcaFilamentLibrary"))
}

fn is_default_local_profile_root(path: &Path) -> bool {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "default" | "local" | "default_user"
            )
        })
}

fn load_source_catalogs(
    roots: &[ApprovedSourceRoot],
) -> Result<BTreeMap<SourceApp, ProfileCatalog>, AppError> {
    let mut roots_by_app = BTreeMap::<SourceApp, Vec<ApprovedSourceRoot>>::new();
    for root in roots {
        roots_by_app
            .entry(root.source_app)
            .or_default()
            .push(root.clone());
    }
    roots_by_app
        .into_iter()
        .map(|(source_app, roots)| Ok((source_app, load_source_catalog(&roots)?)))
        .collect()
}

fn resolve_stored_sources(
    roots: &[ApprovedSourceRoot],
    locators: &BTreeMap<ProfileId, SourceProfileLocator>,
    plan: &MigrationPlan,
) -> Result<BTreeMap<ProfileId, crate::resolver::EffectiveProfile>, AppError> {
    let catalogs = load_source_catalogs(roots)?;
    plan.operations
        .iter()
        .map(|operation| {
            let locator = locators.get(&operation.source_id).ok_or_else(|| {
                AppError::InvalidProfile(format!(
                    "source locator is missing: {}",
                    operation.source_id
                ))
            })?;
            let profile = catalogs
                .get(&locator.source_app)
                .ok_or_else(|| {
                    AppError::InvalidProfile(format!(
                        "source catalog is missing: {}",
                        operation.source_id
                    ))
                })?
                .resolve_name(&locator.profile_name)?;
            Ok((operation.source_id.clone(), profile))
        })
        .collect()
}

fn first_string(values: &BTreeMap<String, Value>, key: &str) -> String {
    values
        .get(key)
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

fn display_name(name: &str) -> String {
    name.split_once(" @")
        .map_or(name, |(display, _)| display)
        .to_owned()
}

fn family_variant(display: &str, vendor: &str, material: &str) -> (String, String) {
    let mut remainder = display.trim();
    if remainder
        .get(..vendor.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(vendor))
    {
        remainder = remainder[vendor.len()..].trim();
    }
    if remainder
        .get(..material.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(material))
    {
        remainder = remainder[material.len()..].trim();
    }
    let family = [vendor, material]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (family, remainder.to_owned())
}

fn reconcile_normal_destinations(
    plan: &mut MigrationPlan,
    outputs: OutputSelection,
    account_root: &Path,
    adapter: &BambuAdapter,
    naming: &NamingOptions,
) -> Result<(), AppError> {
    if !outputs.slicing_presets {
        return Ok(());
    }
    let account_root = account_root
        .canonicalize()
        .map_err(|error| AppError::io(account_root, error))?;
    for operation in &mut plan.operations {
        if operation.action == PlanAction::Block {
            continue;
        }
        let candidate = account_root
            .join("filament")
            .join(format!("{}.json", operation.preset_name));
        let bytes = match std::fs::read(&candidate) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if !outputs.custom_filaments {
                    operation.action = PlanAction::Create;
                    operation.conflict = None;
                    operation.precondition_fingerprint = None;
                }
                continue;
            }
            Err(error) => return Err(AppError::io(&candidate, error)),
        };
        let path = candidate
            .canonicalize()
            .map_err(|error| AppError::io(&candidate, error))?;
        if !path.starts_with(&account_root) || !path.is_file() {
            return Err(AppError::UnsafePath(path));
        }
        let value: Value = serde_json::from_slice(&bytes).map_err(|source| AppError::Json {
            path: path.clone(),
            source,
        })?;
        let existing_name = value.get("name").and_then(Value::as_str).unwrap_or("");
        let existing_fingerprint = material_settings_fingerprint(&value, adapter)?;
        operation.precondition_fingerprint = Some(existing_fingerprint.clone());
        let conflict = if existing_name != operation.preset_name {
            Some(Conflict {
                kind: if existing_name.eq_ignore_ascii_case(&operation.preset_name) {
                    ConflictKind::CaseOnlyName
                } else {
                    ConflictKind::IdentityCollision
                },
                message: format!(
                    "normal preset path contains a different identity: {}",
                    path.display()
                ),
            })
        } else if existing_fingerprint != operation.material_settings_fingerprint {
            Some(Conflict {
                kind: ConflictKind::SettingsMismatch,
                message: "normal preset has different effective settings".to_owned(),
            })
        } else {
            None
        };
        if let Some(conflict) = conflict {
            let choice = naming
                .conflict_decisions
                .iter()
                .find(|decision| {
                    decision.source_id == operation.source_id
                        && decision.printer_id == operation.printer_id
                        && decision.nozzle == operation.nozzle
                })
                .map(|decision| decision.choice);
            let can_update = !outputs.custom_filaments
                || matches!(operation.action, PlanAction::Update | PlanAction::Skip);
            match choice {
                Some(ConflictChoice::Skip) => {
                    operation.action = PlanAction::Skip;
                    operation.conflict = None;
                }
                Some(ConflictChoice::Update) if can_update => {
                    operation.action = PlanAction::Update;
                    operation.conflict = None;
                }
                _ => {
                    operation.action = PlanAction::Block;
                    operation.conflict = Some(conflict);
                }
            }
            continue;
        }
        if !outputs.custom_filaments {
            let compatible = value
                .get("compatible_printers")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    AppError::InvalidProfile(format!(
                        "existing normal preset has no compatible_printers: {}",
                        path.display()
                    ))
                })?;
            operation.action = if compatible
                .iter()
                .any(|item| item.as_str() == Some(&operation.printer_preset_name))
            {
                PlanAction::Skip
            } else {
                PlanAction::AddTarget
            };
            operation.conflict = None;
        }
    }
    Ok(())
}

fn snapshot_destination(
    plan: &MigrationPlan,
    outputs: OutputSelection,
    account_root: &Path,
) -> Result<BTreeMap<PathBuf, ExpectedFileState>, AppError> {
    let account_root = account_root
        .canonicalize()
        .map_err(|error| AppError::io(account_root, error))?;
    let mut paths = BTreeSet::new();
    for operation in &plan.operations {
        if outputs.slicing_presets {
            paths.insert(PathBuf::from("filament").join(format!("{}.json", operation.preset_name)));
        }
        if outputs.custom_filaments {
            let profile_name = format!("{} @{}", operation.ams_name, operation.printer_preset_name);
            let base = PathBuf::from("filament/base");
            paths.insert(base.join(format!("{profile_name}.json")));
            paths.insert(base.join(format!("{profile_name}.info")));
        }
    }
    paths
        .into_iter()
        .map(|relative| {
            if relative.is_absolute()
                || relative
                    .components()
                    .any(|component| !matches!(component, std::path::Component::Normal(_)))
            {
                return Err(AppError::UnsafePath(relative));
            }
            let candidate = account_root.join(&relative);
            let state = match std::fs::read(&candidate) {
                Ok(bytes) => {
                    let path = candidate
                        .canonicalize()
                        .map_err(|error| AppError::io(&candidate, error))?;
                    if !path.starts_with(&account_root) || !path.is_file() {
                        return Err(AppError::UnsafePath(path));
                    }
                    ExpectedFileState::Matches(sha256_bytes(&bytes))
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    ExpectedFileState::Absent
                }
                Err(error) => return Err(AppError::io(&candidate, error)),
            };
            Ok((relative, state))
        })
        .collect()
}

fn load_existing_normal_compatibility(
    plan: &MigrationPlan,
    account_root: &Path,
) -> Result<BTreeMap<String, BTreeSet<String>>, AppError> {
    let account_root = account_root
        .canonicalize()
        .map_err(|error| AppError::io(account_root, error))?;
    let mut result = BTreeMap::new();
    for preset_name in plan
        .operations
        .iter()
        .filter(|operation| operation.action != PlanAction::Block)
        .map(|operation| operation.preset_name.as_str())
        .collect::<BTreeSet<_>>()
    {
        let candidate = account_root
            .join("filament")
            .join(format!("{preset_name}.json"));
        if !candidate.exists() {
            continue;
        }
        let path = candidate
            .canonicalize()
            .map_err(|error| AppError::io(&candidate, error))?;
        if !path.starts_with(&account_root) || !path.is_file() {
            return Err(AppError::UnsafePath(path));
        }
        let bytes = std::fs::read(&path).map_err(|error| AppError::io(&path, error))?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|source| AppError::Json {
            path: path.clone(),
            source,
        })?;
        let compatible = value
            .get("compatible_printers")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AppError::InvalidProfile(format!(
                    "existing normal preset has no compatible_printers: {}",
                    path.display()
                ))
            })?
            .iter()
            .map(|item| {
                item.as_str().map(str::to_owned).ok_or_else(|| {
                    AppError::InvalidProfile(format!(
                        "existing normal preset has invalid compatibility: {}",
                        path.display()
                    ))
                })
            })
            .collect::<Result<_, _>>()?;
        result.insert(preset_name.to_owned(), compatible);
    }
    Ok(result)
}

fn load_existing_sidecars(
    plan: &MigrationPlan,
    account_root: &Path,
) -> Result<BTreeMap<String, InfoSidecar>, AppError> {
    let account_root = account_root
        .canonicalize()
        .map_err(|error| AppError::io(account_root, error))?;
    let mut sidecars = BTreeMap::new();
    for operation in &plan.operations {
        if !matches!(
            operation.action,
            PlanAction::Update | PlanAction::Rename | PlanAction::Replace
        ) {
            continue;
        }
        let profile_name = format!("{} @{}", operation.ams_name, operation.printer_preset_name);
        let candidate = account_root
            .join("filament/base")
            .join(format!("{profile_name}.info"));
        let path = candidate
            .canonicalize()
            .map_err(|error| AppError::io(&candidate, error))?;
        if !path.starts_with(&account_root) || !path.is_file() {
            return Err(AppError::UnsafePath(path));
        }
        let bytes = std::fs::read(&path).map_err(|error| AppError::io(&path, error))?;
        sidecars.insert(operation.id.clone(), InfoSidecar::parse(&bytes)?);
    }
    Ok(sidecars)
}

fn destination_index(
    account_root: &Path,
    adapter: &BambuAdapter,
) -> Result<DestinationIndex, AppError> {
    let base = account_root.join("filament/base");
    if !base.is_dir() {
        return Ok(DestinationIndex::default());
    }
    let mut existing = Vec::new();
    for entry in walkdir::WalkDir::new(&base).follow_links(false) {
        let entry = entry.map_err(|error| {
            AppError::InvalidProfile(format!("destination scan failed: {error}"))
        })?;
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("json")
        {
            continue;
        }
        let bytes =
            std::fs::read(entry.path()).map_err(|error| AppError::io(entry.path(), error))?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|source| AppError::Json {
            path: entry.path().to_path_buf(),
            source,
        })?;
        let Some(name) = value.get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(printer) = value
            .get("compatible_printers")
            .and_then(Value::as_array)
            .and_then(|values| values.first())
            .and_then(Value::as_str)
        else {
            continue;
        };
        let ams_name = name
            .strip_suffix(&format!(" @{printer}"))
            .unwrap_or(name)
            .to_owned();
        let filament_id = value
            .get("filament_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        existing.push(crate::planner::ExistingDestination {
            name: ams_name,
            material_settings_fingerprint: material_settings_fingerprint(&value, adapter)?,
            filament_id,
            printer_preset_name: printer.to_owned(),
        });
    }
    Ok(DestinationIndex::from_existing(existing))
}

fn validate_opaque_id(value: &str) -> Result<(), AppError> {
    if value.is_empty()
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':')
        })
    {
        return Err(AppError::InvalidProfile("opaque id is invalid".to_owned()));
    }
    Ok(())
}

#[derive(Default)]
struct ActiveSynchronizations {
    tokens: Mutex<BTreeMap<String, Arc<AtomicBool>>>,
}

impl ActiveSynchronizations {
    fn start(&self, run_id: &str) -> Result<Arc<AtomicBool>, AppError> {
        validate_opaque_id(run_id)?;
        let mut tokens = self.tokens.lock().map_err(|_| {
            AppError::Conflict("synchronization registry is unavailable".to_owned())
        })?;
        if tokens.contains_key(run_id) {
            return Err(AppError::Conflict(
                "synchronization is already running".to_owned(),
            ));
        }
        let token = Arc::new(AtomicBool::new(false));
        tokens.insert(run_id.to_owned(), token.clone());
        Ok(token)
    }

    fn cancel(&self, run_id: &str) -> Result<(), AppError> {
        validate_opaque_id(run_id)?;
        let tokens = self.tokens.lock().map_err(|_| {
            AppError::Conflict("synchronization registry is unavailable".to_owned())
        })?;
        let token = tokens
            .get(run_id)
            .ok_or_else(|| AppError::Conflict("run is not synchronizing".to_owned()))?;
        token.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn finish(&self, run_id: &str) -> Result<(), AppError> {
        self.tokens
            .lock()
            .map_err(|_| AppError::Conflict("synchronization registry is unavailable".to_owned()))?
            .remove(run_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ActiveSynchronizations, installed_system_profiles};
    use std::sync::atomic::Ordering;

    #[test]
    fn installed_profiles_fall_back_to_executable_resources() {
        let temp = tempfile::tempdir().unwrap();
        let executable = temp.path().join("Bambu Studio/bambu-studio.exe");
        std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
        std::fs::write(&executable, b"stub").unwrap();
        let profiles_root = executable.parent().unwrap().join("resources/profiles");
        let profile_root = profiles_root.join("BBL");
        std::fs::create_dir_all(profile_root.join("filament")).unwrap();
        std::fs::write(profiles_root.join("BBL.json"), b"{}").unwrap();

        let installed =
            installed_system_profiles(&temp.path().join("empty-config"), Some(&executable))
                .unwrap();
        assert_eq!(installed.profile_root, profile_root.canonicalize().unwrap());
        assert_eq!(
            installed.manifest_path,
            profiles_root.join("BBL.json").canonicalize().unwrap()
        );
    }

    #[test]
    fn installed_profiles_find_macos_bundle_resources() {
        let temp = tempfile::tempdir().unwrap();
        let executable = temp
            .path()
            .join("BambuStudio.app/Contents/MacOS/BambuStudio");
        std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
        std::fs::write(&executable, b"stub").unwrap();
        let profiles_root = temp
            .path()
            .join("BambuStudio.app/Contents/Resources/profiles");
        let profile_root = profiles_root.join("BBL");
        std::fs::create_dir_all(profile_root.join("filament")).unwrap();
        std::fs::write(profiles_root.join("BBL.json"), b"{}").unwrap();

        let installed =
            installed_system_profiles(&temp.path().join("empty-config"), Some(&executable))
                .unwrap();
        assert_eq!(installed.profile_root, profile_root.canonicalize().unwrap());
        assert_eq!(
            installed.manifest_path,
            profiles_root.join("BBL.json").canonicalize().unwrap()
        );
    }

    #[test]
    fn active_synchronization_can_be_cancelled_without_service_lock() {
        let active = ActiveSynchronizations::default();
        let token = active.start("run-1").unwrap();

        active.cancel("run-1").unwrap();

        assert!(token.load(Ordering::SeqCst));
        active.finish("run-1").unwrap();
        assert!(active.cancel("run-1").is_err());
    }

    #[test]
    fn debug_demo_state_uses_only_its_temporary_service_workspace() {
        let state = super::AppState::demo().unwrap();
        let workspace = state._demo_workspace.as_ref().unwrap().root();
        let service = state.service.lock().unwrap();

        assert!(state.demo_mode);
        assert!(service.data_root().starts_with(workspace));
        assert!(
            service
                .discover()
                .source_root_ids
                .iter()
                .all(|id| id.starts_with("source:demo:"))
        );
    }
}

pub struct AppState {
    service: Mutex<MigrationService>,
    active_synchronizations: ActiveSynchronizations,
    #[cfg(debug_assertions)]
    demo_mode: bool,
    #[cfg(debug_assertions)]
    _demo_workspace: Option<crate::demo::DemoWorkspace>,
}

impl AppState {
    pub fn current() -> Result<Self, AppError> {
        #[cfg(debug_assertions)]
        if std::env::var_os("SPOOL_LEDGER_DEMO").is_some_and(|value| value == "1") {
            return Self::demo();
        }

        let (config, _) = ServiceConfig::current()?;
        Ok(Self {
            service: Mutex::new(MigrationService::new(config)?),
            active_synchronizations: ActiveSynchronizations::default(),
            #[cfg(debug_assertions)]
            demo_mode: false,
            #[cfg(debug_assertions)]
            _demo_workspace: None,
        })
    }

    #[cfg(debug_assertions)]
    fn demo() -> Result<Self, AppError> {
        let (config, workspace) = crate::demo::create_demo_service_config()?;
        Ok(Self {
            service: Mutex::new(MigrationService::new(config)?),
            active_synchronizations: ActiveSynchronizations::default(),
            demo_mode: true,
            _demo_workspace: Some(workspace),
        })
    }

    #[cfg(debug_assertions)]
    pub fn cleanup_demo_workspace(&self) -> Result<(), AppError> {
        if let Some(workspace) = &self._demo_workspace {
            workspace.cleanup()?;
        }
        Ok(())
    }
}

fn command_error(error: AppError) -> String {
    error.to_string()
}

#[tauri::command]
pub fn discover(state: State<'_, AppState>) -> Result<DiscoveryResponse, String> {
    Ok(state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .discover())
}

#[tauri::command]
pub fn choose_manual_source_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ManualSourceFolderRequest,
) -> Result<Option<DiscoveryResponse>, String> {
    let title = format!(
        "Choose {} {} filament folder",
        source_app_label(request.source_app),
        source_kind_label(request.source_kind)
    );
    let Some(selected) = app.dialog().file().set_title(title).blocking_pick_folder() else {
        return Ok(None);
    };
    let path = selected.into_path().map_err(|error| error.to_string())?;
    let mut service = state.service.lock().map_err(|error| error.to_string())?;
    service
        .add_manual_source_folder(&path, request.source_app, request.source_kind)
        .map_err(command_error)?;
    Ok(Some(service.discover()))
}

#[tauri::command]
pub fn remove_manual_source_folder(
    state: State<'_, AppState>,
    request: RemoveManualSourceFolderRequest,
) -> Result<DiscoveryResponse, String> {
    let mut service = state.service.lock().map_err(|error| error.to_string())?;
    service
        .remove_manual_source_folder(&request.id)
        .map_err(command_error)?;
    Ok(service.discover())
}

#[tauri::command]
pub async fn catalog_sources(
    state: State<'_, AppState>,
    request: CatalogSourcesRequest,
    on_progress: Channel<CatalogProgressEvent>,
) -> Result<SourceCatalogResponse, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .catalog_sources_with_progress(request, |event| {
            let _ = on_progress.send(event);
        })
        .map_err(command_error)
}

#[tauri::command]
pub async fn catalog_targets(
    state: State<'_, AppState>,
    request: CatalogTargetsRequest,
    on_progress: Channel<CatalogProgressEvent>,
) -> Result<TargetCatalogResponse, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .catalog_targets_with_progress(request, |event| {
            let _ = on_progress.send(event);
        })
        .map_err(command_error)
}

#[tauri::command]
pub fn printer_artwork(
    state: State<'_, AppState>,
    catalog_id: String,
    printer_id: String,
) -> Result<Response, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .printer_artwork(&catalog_id, &printer_id)
        .map(Response::new)
        .map_err(command_error)
}

#[tauri::command]
pub fn preview_names(
    state: State<'_, AppState>,
    request: PreviewNamesRequest,
) -> Result<Vec<PreviewNameResult>, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .preview_names(request)
        .map_err(command_error)
}

#[tauri::command]
pub fn build_plan(
    state: State<'_, AppState>,
    request: BuildPlanRequest,
) -> Result<BuildPlanResponse, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .build_plan(request)
        .map_err(command_error)
}

#[tauri::command]
pub fn resolve_plan_dependencies(
    state: State<'_, AppState>,
    request: ResolvePlanDependenciesRequest,
) -> Result<BuildPlanResponse, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .resolve_plan_dependencies(request)
        .map_err(command_error)
}

fn execute_plan_with_backend(
    state: &AppState,
    request: ExecutePlanRequest,
    on_progress: Channel<ExecutionProgressEvent>,
    process: &mut impl ProcessBackend,
    progress_delay_ms: u64,
) -> Result<LocalRunResult, String> {
    let plan_id = request.plan_id.clone();
    let mut clock = SystemClock::default();
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .execute_plan_with_progress(request, process, &mut clock, |phase| {
            let _ = on_progress.send(ExecutionProgressEvent {
                plan_id: plan_id.clone(),
                phase,
            });
            if progress_delay_ms > 0 {
                std::thread::sleep(std::time::Duration::from_millis(progress_delay_ms));
            }
        })
        .map_err(command_error)
}

#[tauri::command]
pub fn execute_plan(
    state: State<'_, AppState>,
    request: ExecutePlanRequest,
    on_progress: Channel<ExecutionProgressEvent>,
) -> Result<LocalRunResult, String> {
    #[cfg(debug_assertions)]
    if state.demo_mode {
        let mut process = crate::demo::DemoProcessBackend;
        return execute_plan_with_backend(&state, request, on_progress, &mut process, 450);
    }

    let mut process = SystemProcessBackend::default();
    execute_plan_with_backend(&state, request, on_progress, &mut process, 0)
}

fn synchronize_run_with_backend(
    state: &AppState,
    request: SynchronizeRunRequest,
    on_progress: Channel<SyncProgressEvent>,
    process: &mut impl ProcessBackend,
    timeout_ms: u64,
    poll_interval_ms: u64,
    progress_delay_ms: u64,
) -> Result<SyncResult, String> {
    let token = state
        .active_synchronizations
        .start(&request.run_id)
        .map_err(command_error)?;
    let mut runtime = FsSyncRuntime::new(token);
    let result = match state.service.lock() {
        Ok(service) => service
            .synchronize_run_with_progress(
                &request.run_id,
                process,
                &mut runtime,
                timeout_ms,
                poll_interval_ms,
                |phase| {
                    let _ = on_progress.send(SyncProgressEvent {
                        run_id: request.run_id.clone(),
                        phase,
                    });
                    if progress_delay_ms > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(progress_delay_ms));
                    }
                },
            )
            .map_err(command_error),
        Err(error) => Err(error.to_string()),
    };
    state
        .active_synchronizations
        .finish(&request.run_id)
        .map_err(command_error)?;
    result
}

#[tauri::command]
pub fn synchronize_run(
    state: State<'_, AppState>,
    request: SynchronizeRunRequest,
    on_progress: Channel<SyncProgressEvent>,
) -> Result<SyncResult, String> {
    #[cfg(debug_assertions)]
    if state.demo_mode {
        let mut process = crate::demo::DemoProcessBackend;
        return synchronize_run_with_backend(&state, request, on_progress, &mut process, 1, 1, 350);
    }

    let mut process = SystemProcessBackend::default();
    synchronize_run_with_backend(&state, request, on_progress, &mut process, 120_000, 500, 0)
}

#[tauri::command]
pub fn cancel_run(state: State<'_, AppState>, run_id: String) -> Result<(), String> {
    state
        .active_synchronizations
        .cancel(&run_id)
        .map_err(command_error)
}

#[tauri::command]
pub fn record_ams_verification(
    state: State<'_, AppState>,
    request: AmsVerificationRequest,
) -> Result<(), String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .record_ams_verification(request)
        .map_err(command_error)
}

#[tauri::command]
pub fn restore_preview(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<RestorePreview, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .restore_preview(&run_id)
        .map_err(command_error)
}

#[tauri::command]
pub fn restore_owned(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<RollbackOutcome, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .restore_owned(&run_id)
        .map_err(command_error)
}
