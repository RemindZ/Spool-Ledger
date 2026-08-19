use crate::AppError;
use crate::discovery::{AccountRoot, DiscoveryService, DiscoverySnapshot, inspect_account};
use crate::model::{AccountEligibility, PrinterTarget, ProfileId, SourceApp, SourceKind};
use crate::naming::{NamingContext, NamingTemplate, ReplacementRuleSpec};
use crate::planner::{
    DestinationIndex, MaterialFingerprintIndex, MigrationPlan, MigrationRequest, MigrationSource,
    MigrationStatus, NamingOptions, PlanAction, Planner, TargetSelection,
};
use crate::receipt::{ReceiptState, RunReceipt};
use crate::resolver::{CatalogRoot, ProfileCatalog};
use crate::sync::{Clock, ProcessBackend, ProcessController, SystemClock, SystemProcessBackend};
use crate::targets::TargetCatalog;
use crate::transaction::{RestorePreview, RollbackOutcome, Transaction, TransactionOptions};
use crate::writer::{
    BambuAdapter, Writer, WriterContext, effective_settings_fingerprint,
    generated_material_settings_fingerprint, material_settings_fingerprint,
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

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
        register_system_source(&mut sources, &discovery.bambu, SourceApp::BambuStudio);
        for account in &discovery.orca_accounts {
            register_user_source(&mut sources, account, SourceApp::OrcaSlicer);
        }
        for account in &discovery.bambu_accounts {
            register_user_source(&mut sources, account, SourceApp::BambuStudio);
        }
        let manifest_path = discovery.bambu.config_root.join("system/BBL/BBL.json");
        let targets = if manifest_path.is_file() {
            vec![ApprovedTargetCatalog {
                id: "target:bambu:installed".to_owned(),
                profile_root: manifest_path
                    .parent()
                    .expect("manifest has parent")
                    .join("filament"),
                manifest_path,
                custom_machine_root: None,
            }]
        } else {
            Vec::new()
        };
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

fn register_system_source(
    sources: &mut Vec<ApprovedSourceRoot>,
    installation: &crate::discovery::Installation,
    source_app: SourceApp,
) {
    let path = installation.config_root.join("system/BBL/filament");
    if path.is_dir() {
        sources.push(ApprovedSourceRoot {
            id: format!("source:{}:system", source_app_id(source_app)),
            path,
            source_app,
            source_kind: SourceKind::FactorySystem,
        });
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub source_root_ids: Vec<String>,
    pub target_catalog_ids: Vec<String>,
    pub accounts: Vec<AccountRoot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogSourcesRequest {
    pub root_ids: Vec<String>,
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
    pub naming: NamingOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildPlanResponse {
    pub plan: MigrationPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutePlanRequest {
    pub plan_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalRunResult {
    pub run_id: String,
    pub plan_id: String,
    pub committed_files: usize,
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

struct CachedSources {
    roots: Vec<ApprovedSourceRoot>,
    sources: BTreeMap<String, MigrationSource>,
    catalog: ProfileCatalog,
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
    target: ApprovedTargetCatalog,
    target_profiles: BTreeMap<String, String>,
    destination_account_id: String,
}

pub struct MigrationService {
    config: ServiceConfig,
    source_catalogs: BTreeMap<String, CachedSources>,
    target_catalogs: BTreeMap<String, CachedTargets>,
    plans: BTreeMap<String, StoredPlan>,
    runs: BTreeMap<String, PathBuf>,
    cancelled_runs: BTreeSet<String>,
}

impl MigrationService {
    pub fn new(mut config: ServiceConfig) -> Result<Self, AppError> {
        canonicalize_config(&mut config)?;
        Ok(Self {
            config,
            source_catalogs: BTreeMap::new(),
            target_catalogs: BTreeMap::new(),
            plans: BTreeMap::new(),
            runs: BTreeMap::new(),
            cancelled_runs: BTreeSet::new(),
        })
    }

    pub fn discover(&self) -> DiscoveryResponse {
        DiscoveryResponse {
            source_root_ids: self
                .config
                .sources
                .iter()
                .map(|item| item.id.clone())
                .collect(),
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
        }
    }

    pub fn catalog_sources(
        &mut self,
        request: CatalogSourcesRequest,
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
        let apps: BTreeSet<_> = roots.iter().map(|root| root.source_app).collect();
        if apps.len() != 1 {
            return Err(AppError::InvalidProfile(
                "one source catalog cannot mix slicer applications".to_owned(),
            ));
        }
        let catalog = load_source_catalog(&roots)?;
        let mut sources = BTreeMap::new();
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
            sources.insert(
                record.id.0.clone(),
                MigrationSource {
                    id: record.id.clone(),
                    name: display,
                    vendor,
                    material,
                    family,
                    variant,
                    source_app: record.source_app,
                    source_kind: record.source_kind,
                    compatible_printers,
                    migration_status: MigrationStatus::New,
                    source_precondition_fingerprint: effective_settings_fingerprint(&effective)?,
                    existing_filament_id,
                },
            );
        }
        let response_sources = sources.values().map(CatalogSource::from).collect();
        let catalog_id = format!("sources:{}", Uuid::new_v4().simple());
        self.source_catalogs.insert(
            catalog_id.clone(),
            CachedSources {
                roots,
                sources,
                catalog,
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
        let catalog = TargetCatalog::load(
            &approved.manifest_path,
            approved.custom_machine_root.as_deref(),
        )?;
        let profiles = ProfileCatalog::load_roots(&[CatalogRoot::system(
            approved.profile_root.clone(),
            SourceApp::BambuStudio,
        )])?;
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

    pub fn build_plan(&mut self, request: BuildPlanRequest) -> Result<BuildPlanResponse, AppError> {
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
        for source in &migration_request.sources {
            let source_effective = source_cache.catalog.resolve_name(&source.id.0)?;
            for target in &migration_request.targets {
                let candidate = format!("Generic {} @BBL {}", source.material, target.printer_code);
                let target_effective =
                    target_cache
                        .profiles
                        .resolve_name(&candidate)
                        .map_err(|_| {
                            AppError::UnsupportedSchema(format!(
                                "target template is missing: {candidate}"
                            ))
                        })?;
                let key = (source.id.clone(), target.printer_preset_name.clone());
                material_fingerprints.insert(
                    key.clone(),
                    generated_material_settings_fingerprint(
                        &source_effective,
                        &target_effective,
                        &adapter,
                    )?,
                );
                target_profile_names.insert(key, candidate);
            }
        }
        let destinations = destination_index(&account.account.path, &adapter)?;
        let plan = Planner::build_with_context(
            &migration_request,
            &destinations,
            &request.naming,
            &material_fingerprints,
        )?;
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
        self.plans.insert(
            plan.id.clone(),
            StoredPlan {
                plan: plan.clone(),
                source_roots: source_cache.roots.clone(),
                target: target_cache.approved.clone(),
                target_profiles,
                destination_account_id: account.account.id.clone(),
            },
        );
        Ok(BuildPlanResponse { plan })
    }

    pub fn execute_plan_with(
        &mut self,
        request: ExecutePlanRequest,
        process: &mut impl ProcessBackend,
        clock: &mut impl Clock,
    ) -> Result<LocalRunResult, AppError> {
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
        let account = self.approved_eligible_account(&stored.destination_account_id)?;
        ProcessController::ensure_stopped(
            process,
            clock,
            self.config.process_close_timeout_ms,
            50,
        )?;
        let sources = load_source_catalog(&stored.source_roots)?;
        let targets = ProfileCatalog::load_roots(&[CatalogRoot::system(
            stored.target.profile_root.clone(),
            SourceApp::BambuStudio,
        )])?;
        let run_id = Uuid::new_v4().simple().to_string();
        if self.cancelled_runs.remove(&run_id) {
            return Err(AppError::Cancelled);
        }
        let staging_root = self.config.data_root.join("staging").join(&run_id);
        let updated_time = chrono::Utc::now().timestamp();
        let staged = Writer::stage(
            &stored.plan,
            &WriterContext {
                sources: &sources,
                targets: &targets,
                target_profiles: stored.target_profiles,
                adapter: BambuAdapter::v2_0_0_56(),
                updated_time,
                existing_sidecars: BTreeMap::new(),
            },
            &staging_root,
        )?;
        let run_root = self.config.data_root.join("runs").join(&run_id);
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
        let outcome = transaction.commit()?;
        let receipt = RunReceipt::from_transaction(&transaction, ReceiptState::Committed)?;
        let receipt_path = run_root.join("receipt.json");
        receipt.save(&receipt_path)?;
        self.runs.insert(run_id.clone(), run_root);
        let _ = std::fs::remove_dir_all(staging_root);
        Ok(LocalRunResult {
            run_id,
            plan_id: stored.plan.id,
            committed_files: outcome.committed,
            receipt_path,
        })
    }

    pub fn restore_preview(&self, run_id: &str) -> Result<RestorePreview, AppError> {
        Transaction::load(&self.known_run_root(run_id)?)?.restore_preview()
    }

    pub fn restore_owned(&mut self, run_id: &str) -> Result<RollbackOutcome, AppError> {
        let run_root = self.known_run_root(run_id)?;
        let mut transaction = Transaction::load(&run_root)?;
        let outcome = transaction.rollback_owned()?;
        let state = if outcome.external_conflicts == 0 {
            ReceiptState::RolledBack
        } else {
            ReceiptState::PartialRollback
        };
        RunReceipt::from_transaction(&transaction, state)?.save(&run_root.join("receipt.json"))?;
        Ok(outcome)
    }

    pub fn cancel_run(&mut self, run_id: &str) -> Result<(), AppError> {
        validate_opaque_id(run_id)?;
        self.cancelled_runs.insert(run_id.to_owned());
        Ok(())
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
        let record = AmsVerificationRecord {
            run_id: request.run_id,
            operation_ids: request.operation_ids,
            note: request.note,
            recorded_at: chrono::Utc::now().timestamp(),
            operator_verified: true,
        };
        let path = run_root.join("ams-verification.json");
        let mut bytes = serde_json::to_vec_pretty(&record)
            .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes).map_err(|error| AppError::io(path, error))
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

fn load_source_catalog(roots: &[ApprovedSourceRoot]) -> Result<ProfileCatalog, AppError> {
    let roots: Vec<_> = roots
        .iter()
        .map(|root| match root.source_kind {
            SourceKind::FactorySystem => CatalogRoot::system(root.path.clone(), root.source_app),
            SourceKind::UserCustom => CatalogRoot::user(root.path.clone(), root.source_app),
        })
        .collect();
    ProfileCatalog::load_roots(&roots)
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

pub struct AppState {
    service: Mutex<MigrationService>,
}

impl AppState {
    pub fn current() -> Result<Self, AppError> {
        let (config, _) = ServiceConfig::current()?;
        Ok(Self {
            service: Mutex::new(MigrationService::new(config)?),
        })
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
pub fn catalog_sources(
    state: State<'_, AppState>,
    request: CatalogSourcesRequest,
) -> Result<SourceCatalogResponse, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .catalog_sources(request)
        .map_err(command_error)
}

#[tauri::command]
pub fn catalog_targets(
    state: State<'_, AppState>,
    request: CatalogTargetsRequest,
) -> Result<TargetCatalogResponse, String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .catalog_targets(request)
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
pub fn execute_plan(
    state: State<'_, AppState>,
    request: ExecutePlanRequest,
) -> Result<LocalRunResult, String> {
    let mut process = SystemProcessBackend::default();
    let mut clock = SystemClock::default();
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .execute_plan_with(request, &mut process, &mut clock)
        .map_err(command_error)
}

#[tauri::command]
pub fn cancel_run(state: State<'_, AppState>, run_id: String) -> Result<(), String> {
    state
        .service
        .lock()
        .map_err(|error| error.to_string())?
        .cancel_run(&run_id)
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
