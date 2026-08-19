use bambu_filament_migrator::commands::{
    AmsVerificationRequest, ApprovedAccount, ApprovedSourceRoot, ApprovedTargetCatalog,
    BuildPlanRequest, CatalogSourcesRequest, CatalogTargetsRequest, ExecutePlanRequest,
    MigrationService, NozzleSelection, PreviewNameRow, PreviewNamesRequest, ServiceConfig,
    SyncPhase,
};
use bambu_filament_migrator::discovery::inspect_account;
use bambu_filament_migrator::model::{EvidenceLevel, ProfileId, SourceApp, SourceKind};
use bambu_filament_migrator::naming::{
    ConditionField, ReplacementRuleSpec, RuleConditionSpec, RulePatternKind,
};
use bambu_filament_migrator::planner::{NameOverride, NamingOptions};
use bambu_filament_migrator::receipt::RunReceipt;
use bambu_filament_migrator::sync::{Clock, ProcessBackend, SyncRuntime};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic")
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.unwrap();
        let relative = entry.path().strip_prefix(source).unwrap();
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(target).unwrap();
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

struct FixtureService {
    _temp: tempfile::TempDir,
    service: MigrationService,
    source_root: PathBuf,
    destination: PathBuf,
}

fn fixture_service(eligible: bool) -> FixtureService {
    let temp = tempfile::tempdir().unwrap();
    let source_root = temp.path().join("orca-filament");
    let target_root = temp.path().join("bambu-system");
    copy_tree(&fixtures().join("orca-system/filament"), &source_root);
    copy_tree(&fixtures().join("bambu-system"), &target_root);
    let destination = temp.path().join("account/2182110758");
    std::fs::create_dir_all(destination.join("filament")).unwrap();
    if eligible {
        std::fs::write(
            destination.join("filament/existing.json"),
            br#"{"name":"Existing PLA","filament_settings_id":["Existing PLA"]}"#,
        )
        .unwrap();
    }
    let account = inspect_account(&destination).unwrap();
    let executable = temp.path().join("BambuStudio");
    std::fs::write(&executable, b"synthetic executable").unwrap();
    let config = ServiceConfig {
        sources: vec![ApprovedSourceRoot {
            id: "source:orca:system".to_owned(),
            path: source_root.clone(),
            source_app: SourceApp::OrcaSlicer,
            source_kind: SourceKind::FactorySystem,
        }],
        targets: vec![ApprovedTargetCatalog {
            id: "target:bambu".to_owned(),
            manifest_path: target_root.join("BBL.json"),
            profile_root: target_root.join("filament"),
            custom_machine_root: None,
        }],
        accounts: vec![ApprovedAccount {
            account,
            bambu_executable: Some(executable),
        }],
        data_root: temp.path().join("app-data"),
        process_close_timeout_ms: 200,
    };
    FixtureService {
        service: MigrationService::new(config).unwrap(),
        _temp: temp,
        source_root,
        destination,
    }
}

fn build_plan(fixture: &mut FixtureService) -> String {
    let sources = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:orca:system".to_owned()],
        })
        .unwrap();
    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();
    assert_eq!(targets.printers.len(), 1);
    fixture
        .service
        .build_plan(BuildPlanRequest {
            source_catalog_id: sources.catalog_id,
            source_ids: vec!["Northstar PLA Aurora @BBL X1C".to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "2182110758".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            naming: NamingOptions::default(),
        })
        .unwrap()
        .plan
        .id
}

#[derive(Default)]
struct FakeClock(u64);

impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.0
    }

    fn sleep_ms(&mut self, duration: u64) {
        self.0 += duration;
    }
}

struct FakeProcess {
    checks: VecDeque<Vec<u32>>,
}

impl ProcessBackend for FakeProcess {
    fn bambu_processes(&mut self) -> Result<Vec<u32>, String> {
        Ok(self.checks.pop_front().unwrap_or_default())
    }

    fn request_graceful_close(&mut self, _process_ids: &[u32]) -> Result<bool, String> {
        Ok(true)
    }

    fn launch(&mut self, _executable: &Path) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Default)]
struct SyncProcess {
    launched: Vec<PathBuf>,
}

impl ProcessBackend for SyncProcess {
    fn bambu_processes(&mut self) -> Result<Vec<u32>, String> {
        Ok(Vec::new())
    }

    fn request_graceful_close(&mut self, _process_ids: &[u32]) -> Result<bool, String> {
        Ok(true)
    }

    fn launch(&mut self, executable: &Path) -> Result<(), String> {
        self.launched.push(executable.to_path_buf());
        Ok(())
    }
}

struct FakeSyncRuntime {
    now: u64,
    snapshots: BTreeMap<PathBuf, VecDeque<Option<Vec<u8>>>>,
}

impl SyncRuntime for FakeSyncRuntime {
    fn read(&mut self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        Ok(self
            .snapshots
            .get_mut(path)
            .and_then(VecDeque::pop_front)
            .flatten())
    }

    fn now_ms(&self) -> u64 {
        self.now
    }

    fn sleep_ms(&mut self, duration: u64) {
        self.now += duration;
    }

    fn is_cancelled(&self) -> bool {
        false
    }
}

#[test]
fn command_requests_reject_arbitrary_paths_and_unapproved_root_ids() {
    let request = serde_json::from_value::<CatalogSourcesRequest>(serde_json::json!({
        "root_ids": ["source:orca:system"],
        "path": "C:/arbitrary"
    }));
    assert!(request.is_err());

    let mut fixture = fixture_service(true);
    let error = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:not-approved".to_owned()],
        })
        .unwrap_err();
    assert!(error.to_string().contains("approved"));
}

#[test]
fn preview_and_plan_share_rules_conditions_and_row_overrides() {
    let rule = ReplacementRuleSpec {
        kind: RulePatternKind::Wildcard,
        pattern: "Northstar PLA *".to_owned(),
        replacement: "$1".to_owned(),
        case_sensitive: false,
        condition: Some(RuleConditionSpec {
            field: ConditionField::SourceApp,
            value: "orca_slicer".to_owned(),
        }),
    };
    let mut fixture = fixture_service(true);
    let preview = fixture
        .service
        .preview_names(PreviewNamesRequest {
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            preset_rules: vec![],
            ams_rules: vec![rule.clone()],
            rows: vec![PreviewNameRow {
                source_name: "Northstar PLA Aurora".to_owned(),
                vendor: "Northstar".to_owned(),
                material: "PLA".to_owned(),
                family: "Northstar".to_owned(),
                variant: "Aurora".to_owned(),
                source_app: SourceApp::OrcaSlicer,
                source_kind: SourceKind::FactorySystem,
                printer: "Bambu Lab H2C".to_owned(),
                printer_code: "H2C".to_owned(),
                nozzle: "0.4".to_owned(),
            }],
        })
        .unwrap();
    assert_eq!(preview[0].ams_before, "Northstar PLA Aurora");
    assert_eq!(preview[0].ams_name, "Aurora");

    let sources = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:orca:system".to_owned()],
        })
        .unwrap();
    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();
    let response = fixture
        .service
        .build_plan(BuildPlanRequest {
            source_catalog_id: sources.catalog_id,
            source_ids: vec!["Northstar PLA Aurora @BBL X1C".to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "2182110758".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            naming: NamingOptions {
                preset_rules: vec![],
                ams_rules: vec![rule],
                overrides: vec![NameOverride {
                    source_id: ProfileId::new("Northstar PLA Aurora @BBL X1C"),
                    printer_id: "official:H2C".to_owned(),
                    nozzle: "0.4".to_owned(),
                    preset_name: Some("Aurora tuned H2C".to_owned()),
                    ams_name: None,
                }],
            },
        })
        .unwrap();
    assert_eq!(response.plan.operations[0].preset_name, "Aurora tuned H2C");
    assert_eq!(response.plan.operations[0].ams_name, "Aurora");
}

#[test]
fn ineligible_destination_account_is_rejected_before_plan_creation() {
    let mut fixture = fixture_service(false);
    let sources = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:orca:system".to_owned()],
        })
        .unwrap();
    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();
    let error = fixture
        .service
        .build_plan(BuildPlanRequest {
            source_catalog_id: sources.catalog_id,
            source_ids: vec!["Northstar PLA Aurora @BBL X1C".to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "2182110758".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            naming: NamingOptions::default(),
        })
        .unwrap_err();
    assert!(error.to_string().contains("eligible"));
    assert_eq!(
        std::fs::read_dir(fixture.destination.join("filament"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn stale_plan_and_changed_source_hash_are_rejected_without_destination_writes() {
    let mut fixture = fixture_service(true);
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    let stale = fixture.service.execute_plan_with(
        ExecutePlanRequest {
            plan_id: "missing-plan".to_owned(),
        },
        &mut process,
        &mut clock,
    );
    assert!(stale.unwrap_err().to_string().contains("stale"));

    let plan_id = build_plan(&mut fixture);
    let source = fixture
        .source_root
        .join("Northstar/Northstar PLA Aurora @BBL X1C.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&source).unwrap()).unwrap();
    value["fan_min_speed"] = serde_json::json!(["80"]);
    std::fs::write(&source, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let before = std::fs::read(fixture.destination.join("filament/existing.json")).unwrap();
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let error = fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap_err();
    assert!(error.to_string().contains("fingerprint"));
    assert_eq!(
        std::fs::read(fixture.destination.join("filament/existing.json")).unwrap(),
        before
    );
}

#[test]
fn generated_flat_destination_is_semantically_equivalent_on_second_plan() {
    let mut fixture = fixture_service(true);
    let plan_id = build_plan(&mut fixture);
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap();

    let sources = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:orca:system".to_owned()],
        })
        .unwrap();
    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();
    let second = fixture
        .service
        .build_plan(BuildPlanRequest {
            source_catalog_id: sources.catalog_id,
            source_ids: vec!["Northstar PLA Aurora @BBL X1C".to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "2182110758".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            naming: NamingOptions::default(),
        })
        .unwrap();
    assert_eq!(second.plan.operations.len(), 1);
    assert_eq!(second.plan.operations[0].action.as_str(), "skip");
}

#[test]
fn changed_material_setting_in_flat_destination_blocks_second_plan() {
    let mut fixture = fixture_service(true);
    let plan_id = build_plan(&mut fixture);
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap();

    let destination = fixture
        .destination
        .join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&destination).unwrap()).unwrap();
    value["filament_cost"] = serde_json::json!(["999"]);
    std::fs::write(&destination, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    let sources = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:orca:system".to_owned()],
        })
        .unwrap();
    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();
    let second = fixture
        .service
        .build_plan(BuildPlanRequest {
            source_catalog_id: sources.catalog_id,
            source_ids: vec!["Northstar PLA Aurora @BBL X1C".to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "2182110758".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            naming: NamingOptions::default(),
        })
        .unwrap();
    assert_eq!(second.plan.operations.len(), 1);
    assert_eq!(second.plan.operations[0].action.as_str(), "block");
}

#[test]
fn ams_verification_cannot_outrun_cloud_assignment_evidence() {
    let mut fixture = fixture_service(true);
    let plan_id = build_plan(&mut fixture);
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    let local = fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap();
    let transaction = bambu_filament_migrator::transaction::Transaction::load(
        &fixture.service.data_root().join("runs").join(&local.run_id),
    )
    .unwrap();
    let operation_id = transaction.journal().entries[0].operation_ids[0].clone();

    let error = fixture
        .service
        .record_ams_verification(AmsVerificationRequest {
            run_id: local.run_id,
            operation_ids: vec![operation_id],
            note: "not actually checked".to_owned(),
        })
        .unwrap_err();

    assert!(error.to_string().contains("cloud"));
}

#[test]
fn synchronization_launches_bambu_monitors_journaled_sidecars_and_updates_receipt() {
    let mut fixture = fixture_service(true);
    let plan_id = build_plan(&mut fixture);
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    let local = fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap();
    let info_path = fixture
        .destination
        .join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.info")
        .canonicalize()
        .unwrap();
    let initial = std::fs::read(&info_path).unwrap();
    let cloud = String::from_utf8(initial.clone())
        .unwrap()
        .replace("setting_id = \n", "setting_id = PFUSunique123\n")
        .into_bytes();
    let mut timeout_runtime = FakeSyncRuntime {
        now: 0,
        snapshots: BTreeMap::from([(
            info_path.clone(),
            VecDeque::from([Some(initial.clone()), Some(initial.clone())]),
        )]),
    };
    let timeout = fixture
        .service
        .synchronize_run_with(
            &local.run_id,
            &mut SyncProcess::default(),
            &mut timeout_runtime,
            100,
            100,
        )
        .unwrap();
    assert!(timeout.timed_out);
    assert_eq!(timeout.highest_evidence, EvidenceLevel::CreatedLocal);

    let mut runtime = FakeSyncRuntime {
        now: 0,
        snapshots: BTreeMap::from([(info_path, VecDeque::from([Some(initial), Some(cloud)]))]),
    };
    let mut sync_process = SyncProcess::default();
    let mut phases = Vec::new();

    let synchronization = fixture
        .service
        .synchronize_run_with_progress(
            &local.run_id,
            &mut sync_process,
            &mut runtime,
            1_000,
            100,
            |phase| phases.push(phase),
        )
        .unwrap();

    assert_eq!(
        phases,
        vec![
            SyncPhase::Launching,
            SyncPhase::Monitoring,
            SyncPhase::Finished
        ]
    );
    assert_eq!(sync_process.launched.len(), 1);
    assert_eq!(
        synchronization.highest_evidence,
        EvidenceLevel::CloudIdAssigned
    );
    assert!(!synchronization.timed_out);
    let receipt = RunReceipt::load(&local.receipt_path).unwrap();
    assert_eq!(receipt.synchronization.as_ref(), Some(&synchronization));

    let verified_operation = synchronization.observations[0].operation_id.clone();
    fixture
        .service
        .record_ams_verification(AmsVerificationRequest {
            run_id: local.run_id.clone(),
            operation_ids: vec![verified_operation.clone()],
            note: "verified after restart".to_owned(),
        })
        .unwrap();
    let verified_receipt = RunReceipt::load(&local.receipt_path).unwrap();
    assert_eq!(
        verified_receipt.ams_verification.unwrap().operation_ids,
        vec![verified_operation]
    );

    fixture.service.restore_owned(&local.run_id).unwrap();
    let restored_receipt = RunReceipt::load(&local.receipt_path).unwrap();
    assert_eq!(
        restored_receipt.synchronization.as_ref(),
        Some(&synchronization)
    );
    assert!(restored_receipt.ams_verification.is_some());
}

#[test]
fn running_bambu_blocks_execution_before_staging_or_writes() {
    let mut fixture = fixture_service(true);
    let plan_id = build_plan(&mut fixture);
    let before = std::fs::read(fixture.destination.join("filament/existing.json")).unwrap();
    let mut process = FakeProcess {
        checks: std::iter::repeat_n(vec![9], 10).collect(),
    };
    let mut clock = FakeClock::default();
    let error = fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap_err();
    assert!(error.to_string().contains("still running"));
    assert_eq!(
        std::fs::read(fixture.destination.join("filament/existing.json")).unwrap(),
        before
    );
    assert!(!fixture.service.data_root().join("runs").exists());
}

#[test]
fn execute_commits_only_to_approved_account_and_restore_is_run_id_based() {
    let mut fixture = fixture_service(true);
    let plan_id = build_plan(&mut fixture);
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    let result = fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap();
    assert_eq!(result.committed_files, 3);
    assert!(
        fixture
            .destination
            .join("filament/Aurora - H2C.json")
            .is_file()
    );
    assert!(
        fixture
            .destination
            .join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json")
            .is_file()
    );
    let preview = fixture.service.restore_preview(&result.run_id).unwrap();
    assert_eq!(preview.paths.len(), 3);
    let rollback = fixture.service.restore_owned(&result.run_id).unwrap();
    assert_eq!(rollback.rolled_back, 3);
    assert!(
        !fixture
            .destination
            .join("filament/Aurora - H2C.json")
            .exists()
    );
}
