use bambu_filament_migrator::commands::{
    AmsVerificationRequest, ApprovedAccount, ApprovedSourceRoot, ApprovedTargetCatalog,
    BuildPlanRequest, CatalogProgressPhase, CatalogSourcesRequest, CatalogTargetsRequest,
    ExecutePlanRequest, ExecutionPhase, ManualSourceFolderRequest, MigrationService,
    NozzleSelection, OutputSelection, PreviewNameRow, PreviewNamesRequest,
    ResolvePlanDependenciesRequest, ServiceConfig, SyncPhase, TargetTemplateDecision,
    TargetTemplateDecisionAction,
};
use bambu_filament_migrator::discovery::inspect_account;
use bambu_filament_migrator::model::{EvidenceLevel, ProfileId, SourceApp, SourceKind};
use bambu_filament_migrator::naming::{
    ConditionField, ReplacementRuleSpec, RuleConditionSpec, RulePatternKind,
};
use bambu_filament_migrator::planner::{
    ConflictChoice, ConflictDecision, NameOverride, NamingOptions,
};
use bambu_filament_migrator::profiles::{InfoSidecar, SyncAction};
use bambu_filament_migrator::receipt::RunReceipt;
use bambu_filament_migrator::sync::{Clock, ProcessBackend, SyncRuntime};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};

const SOURCE_PROFILE_ID: &str = "profile:orca:Northstar PLA Aurora @BBL X1C";

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

fn profile_tree_fingerprints(roots: &[PathBuf]) -> BTreeMap<PathBuf, String> {
    let mut paths: Vec<_> = roots
        .iter()
        .filter(|root| root.exists())
        .flat_map(|root| {
            walkdir::WalkDir::new(root)
                .follow_links(false)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_file())
                .map(|entry| entry.into_path())
                .collect::<Vec<_>>()
        })
        .collect();
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap();
            (path, bambu_filament_migrator::sync::sha256_bytes(&bytes))
        })
        .collect()
}

struct FixtureService {
    _temp: tempfile::TempDir,
    service: MigrationService,
    source_root: PathBuf,
    bambu_source_root: PathBuf,
    target_root: PathBuf,
    destination: PathBuf,
}

fn fixture_service(eligible: bool) -> FixtureService {
    let temp = tempfile::tempdir().unwrap();
    let source_root = temp.path().join("orca-filament");
    let bambu_source_root = temp.path().join("bambu-user-filament");
    let target_root = temp.path().join("bambu-system");
    copy_tree(&fixtures().join("orca-system/filament"), &source_root);
    std::fs::create_dir_all(&bambu_source_root).unwrap();
    copy_tree(&fixtures().join("bambu-system"), &target_root);
    let custom_machine_root = temp.path().join("bambu-custom-machine");
    std::fs::create_dir_all(&custom_machine_root).unwrap();
    std::fs::write(
        custom_machine_root.join("Workshop CoreXY 0.4 nozzle.json"),
        br#"{"name":"Workshop CoreXY 0.4 nozzle","nozzle_diameter":["0.4"]}"#,
    )
    .unwrap();
    let destination = temp.path().join("account/0000000000");
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
        sources: vec![
            ApprovedSourceRoot {
                id: "source:orca:system".to_owned(),
                path: source_root.clone(),
                source_app: SourceApp::OrcaSlicer,
                source_kind: SourceKind::FactorySystem,
            },
            ApprovedSourceRoot {
                id: "source:bambu:user".to_owned(),
                path: bambu_source_root.clone(),
                source_app: SourceApp::BambuStudio,
                source_kind: SourceKind::UserCustom,
            },
        ],
        targets: vec![ApprovedTargetCatalog {
            id: "target:bambu".to_owned(),
            manifest_path: target_root.join("BBL.json"),
            profile_root: target_root.join("filament"),
            custom_machine_root: Some(custom_machine_root),
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
        bambu_source_root,
        target_root,
        destination,
    }
}

fn build_plan(fixture: &mut FixtureService) -> String {
    build_plan_with_outputs(fixture, OutputSelection::default())
}

fn build_plan_with_outputs(fixture: &mut FixtureService, outputs: OutputSelection) -> String {
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
            source_ids: vec![SOURCE_PROFILE_ID.to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs,
            naming: NamingOptions::default(),
        })
        .unwrap()
        .into_plan()
        .unwrap()
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
fn discovery_reports_backend_derived_installations_and_platform() {
    let fixture = fixture_service(true);

    let discovery = fixture.service.discover();

    assert!(discovery.orca_slicer_detected);
    assert!(discovery.bambu_studio_detected);
    assert_eq!(discovery.platform, std::env::consts::OS);
}

#[test]
fn source_catalog_reports_loading_resolving_and_completion_counts() {
    let mut fixture = fixture_service(true);
    let mut events = Vec::new();

    let response = fixture
        .service
        .catalog_sources_with_progress(
            CatalogSourcesRequest {
                root_ids: vec!["source:orca:system".to_owned()],
            },
            |event| events.push(event),
        )
        .unwrap();

    assert!(!response.sources.is_empty());
    for phase in [
        CatalogProgressPhase::Loading,
        CatalogProgressPhase::Resolving,
        CatalogProgressPhase::Finished,
    ] {
        assert!(events.iter().any(|event| event.phase == phase));
    }
    for phase in [
        CatalogProgressPhase::Loading,
        CatalogProgressPhase::Resolving,
    ] {
        let phase_events: Vec<_> = events.iter().filter(|event| event.phase == phase).collect();
        assert_eq!(phase_events.first().unwrap().processed, 0);
        assert_eq!(
            phase_events.last().unwrap().processed,
            phase_events.last().unwrap().total
        );
    }
}

#[test]
fn target_catalog_reports_loading_resolving_and_completion_counts() {
    let mut fixture = fixture_service(true);
    let mut events = Vec::new();

    let response = fixture
        .service
        .catalog_targets_with_progress(
            CatalogTargetsRequest {
                catalog_id: "target:bambu".to_owned(),
                show_custom: false,
            },
            |event| events.push(event),
        )
        .unwrap();

    assert!(!response.printers.is_empty());
    for phase in [
        CatalogProgressPhase::Loading,
        CatalogProgressPhase::Resolving,
        CatalogProgressPhase::Finished,
    ] {
        assert!(events.iter().any(|event| event.phase == phase));
    }
    for phase in [
        CatalogProgressPhase::Loading,
        CatalogProgressPhase::Resolving,
    ] {
        let phase_events: Vec<_> = events.iter().filter(|event| event.phase == phase).collect();
        assert_eq!(phase_events.first().unwrap().processed, 0);
        assert_eq!(
            phase_events.last().unwrap().processed,
            phase_events.last().unwrap().total
        );
    }
}

#[test]
fn broad_generic_template_remains_valid_when_output_expands_nozzle_compatibility() {
    let mut fixture = fixture_service(true);
    let path = fixture
        .target_root
        .join("filament/Generic PLA @BBL H2C.json");
    let mut profile: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    profile["compatible_printers"] =
        serde_json::json!(["Bambu Lab H2C 0.6 nozzle", "Bambu Lab H2C 0.8 nozzle"]);
    std::fs::write(path, serde_json::to_vec_pretty(&profile).unwrap()).unwrap();

    let plan_id = build_plan(&mut fixture);

    assert!(!plan_id.is_empty());
}

#[test]
fn printer_artwork_is_available_only_through_cached_opaque_ids() {
    let mut fixture = fixture_service(true);
    let artwork = b"\x89PNG\r\n\x1a\nsynthetic-printer-cover";
    std::fs::write(fixture.target_root.join("Bambu Lab H2C_cover.png"), artwork).unwrap();

    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();
    let serialized = serde_json::to_value(&targets.printers[0]).unwrap();

    assert_eq!(serialized["artwork_available"], true);
    assert!(serialized.get("artwork_path").is_none());
    assert_eq!(
        fixture
            .service
            .printer_artwork(&targets.catalog_id, "official:H2C")
            .unwrap(),
        artwork
    );
    assert!(
        fixture
            .service
            .printer_artwork(&targets.catalog_id, "../../escape")
            .is_err()
    );
}

#[test]
fn pet_cf_fallback_is_returned_as_typed_target_dependency() {
    let mut fixture = fixture_service(true);
    std::fs::write(
        fixture.source_root.join("Elegoo PET-CF @System.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "type": "filament",
            "name": "Elegoo PET-CF @System",
            "instantiation": "true",
            "filament_vendor": ["Elegoo"],
            "filament_type": ["PET-CF"],
            "nozzle_temperature": ["270"],
            "nozzle_temperature_initial_layer": ["270"]
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::write(
        fixture.target_root.join("filament/Bambu PET-CF @base.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "type": "filament",
            "name": "Bambu PET-CF @base",
            "inherits": "fdm_filament_pla",
            "instantiation": "false",
            "filament_vendor": ["Bambu Lab"],
            "filament_type": ["PET-CF"]
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::write(
        fixture
            .target_root
            .join("filament/Bambu PET-CF @BBL H2C.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "type": "filament",
            "name": "Bambu PET-CF @BBL H2C",
            "inherits": "Bambu PET-CF @base",
            "instantiation": "true",
            "compatible_printers": ["Bambu Lab H2C 0.4 nozzle"],
            "filament_extruder_variant": [
                "Direct Drive Standard",
                "Direct Drive High Flow",
                "Direct Drive E3D High Flow"
            ],
            "nozzle_temperature": ["270", "270", "270"],
            "nozzle_temperature_initial_layer": ["270", "270", "270"]
        }))
        .unwrap(),
    )
    .unwrap();
    let sources = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:orca:system".to_owned()],
        })
        .unwrap();
    let source = sources
        .sources
        .iter()
        .find(|source| source.vendor == "Elegoo" && source.material == "PET-CF")
        .unwrap();
    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();

    let request = BuildPlanRequest {
        source_catalog_id: sources.catalog_id,
        source_ids: vec![source.id.0.clone()],
        target_catalog_id: targets.catalog_id,
        nozzles: vec![NozzleSelection {
            printer_id: "official:H2C".to_owned(),
            diameters: vec!["0.4".to_owned()],
        }],
        destination_account_id: "0000000000".to_owned(),
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        outputs: OutputSelection::default(),
        naming: NamingOptions::default(),
    };
    let response = fixture.service.build_plan(request.clone()).unwrap();
    let value = serde_json::to_value(response).unwrap();

    assert_eq!(value["status"], "needs_resolution");
    assert_eq!(
        value["issues"][0]["expected_name"],
        "Generic PET-CF @BBL H2C"
    );
    assert_eq!(
        value["issues"][0]["installed_candidates"][0]["profile_name"],
        "Bambu PET-CF @BBL H2C"
    );
    assert_eq!(
        value["issues"][0]["affected_sources"][0]["name"],
        "Elegoo PET-CF"
    );

    let plan = fixture
        .service
        .resolve_plan_dependencies(ResolvePlanDependenciesRequest {
            request,
            decisions: vec![TargetTemplateDecision {
                issue_id: value["issues"][0]["id"].as_str().unwrap().to_owned(),
                action: TargetTemplateDecisionAction::UseInstalled,
                profile_name: Some("Bambu PET-CF @BBL H2C".to_owned()),
                source_id: None,
            }],
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(plan.operations.len(), 1);
    assert_eq!(plan.operations[0].action.as_str(), "create");
}

#[test]
fn missing_target_dependencies_aggregate_every_affected_source() {
    let mut fixture = fixture_service(true);
    for (name, vendor) in [
        ("Vendor One ULTEM", "Vendor One"),
        ("Vendor Two ULTEM", "Vendor Two"),
    ] {
        std::fs::write(
            fixture.source_root.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&serde_json::json!({
                "type": "filament",
                "name": name,
                "instantiation": "true",
                "filament_vendor": [vendor],
                "filament_type": ["ULTEM"],
                "nozzle_temperature": ["380"],
                "nozzle_temperature_initial_layer": ["380"]
            }))
            .unwrap(),
        )
        .unwrap();
    }
    let sources = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:orca:system".to_owned()],
        })
        .unwrap();
    let source_ids: Vec<_> = sources
        .sources
        .iter()
        .filter(|source| source.material == "ULTEM")
        .map(|source| source.id.0.clone())
        .collect();
    assert_eq!(source_ids.len(), 2);
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
            source_ids,
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
            naming: NamingOptions::default(),
        })
        .unwrap();
    let value = serde_json::to_value(response).unwrap();

    assert_eq!(value["status"], "needs_resolution");
    assert_eq!(value["issues"].as_array().unwrap().len(), 1);
    assert_eq!(
        value["issues"][0]["affected_sources"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        value["issues"][0]["installed_candidates"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        value["issues"][0]["source_candidates"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn ambiguous_target_candidates_require_a_current_approved_decision() {
    let mut fixture = fixture_service(true);
    std::fs::write(
        fixture.source_root.join("Vendor PET-CF.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "type": "filament",
            "name": "Vendor PET-CF",
            "instantiation": "true",
            "filament_vendor": ["Vendor"],
            "filament_type": ["PET-CF"],
            "nozzle_temperature": ["270"],
            "nozzle_temperature_initial_layer": ["270"]
        }))
        .unwrap(),
    )
    .unwrap();
    for name in ["Bambu PET-CF @BBL H2C 0.4 nozzle", "Fiber PET-CF @BBL H2C"] {
        std::fs::write(
            fixture
                .target_root
                .join("filament")
                .join(format!("{name}.json")),
            serde_json::to_vec_pretty(&serde_json::json!({
                "type": "filament",
                "name": name,
                "instantiation": "true",
                "filament_vendor": ["Bambu Lab"],
                "filament_type": ["PET-CF"],
                "compatible_printers": ["Bambu Lab H2C 0.4 nozzle"],
                "nozzle_temperature": ["270"],
                "nozzle_temperature_initial_layer": ["270"]
            }))
            .unwrap(),
        )
        .unwrap();
    }
    let sources = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:orca:system".to_owned()],
        })
        .unwrap();
    let source = sources
        .sources
        .iter()
        .find(|source| source.name == "Vendor PET-CF")
        .unwrap();
    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();
    let request = BuildPlanRequest {
        source_catalog_id: sources.catalog_id,
        source_ids: vec![source.id.0.clone()],
        target_catalog_id: targets.catalog_id,
        nozzles: vec![NozzleSelection {
            printer_id: "official:H2C".to_owned(),
            diameters: vec!["0.4".to_owned()],
        }],
        destination_account_id: "0000000000".to_owned(),
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        outputs: OutputSelection::default(),
        naming: NamingOptions::default(),
    };
    let response = fixture.service.build_plan(request.clone()).unwrap();
    let value = serde_json::to_value(response).unwrap();
    let issue_id = value["issues"][0]["id"].as_str().unwrap();
    let candidates = value["issues"][0]["installed_candidates"]
        .as_array()
        .unwrap();

    assert_eq!(candidates.len(), 2);
    assert_eq!(
        candidates[0]["profile_name"],
        "Bambu PET-CF @BBL H2C 0.4 nozzle"
    );
    assert_eq!(
        candidates
            .iter()
            .filter(|candidate| candidate["recommended"] == true)
            .count(),
        1
    );

    let stale = fixture
        .service
        .resolve_plan_dependencies(ResolvePlanDependenciesRequest {
            request: request.clone(),
            decisions: vec![TargetTemplateDecision {
                issue_id: "stale-issue".to_owned(),
                action: TargetTemplateDecisionAction::UseInstalled,
                profile_name: Some("Bambu PET-CF @BBL H2C 0.4 nozzle".to_owned()),
                source_id: None,
            }],
        })
        .unwrap_err();
    assert!(stale.to_string().contains("stale or unrelated"));

    let arbitrary = fixture
        .service
        .resolve_plan_dependencies(ResolvePlanDependenciesRequest {
            request,
            decisions: vec![TargetTemplateDecision {
                issue_id: issue_id.to_owned(),
                action: TargetTemplateDecisionAction::UseInstalled,
                profile_name: Some("Generic PLA @BBL H2C".to_owned()),
                source_id: None,
            }],
        })
        .unwrap_err();
    assert!(arbitrary.to_string().contains("not an approved candidate"));
}

#[test]
fn validated_bambu_source_can_supply_a_missing_target_template() {
    let mut fixture = fixture_service(true);
    std::fs::write(
        fixture.source_root.join("Vendor PEKK-CF.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "type": "filament",
            "name": "Vendor PEKK-CF",
            "instantiation": "true",
            "filament_vendor": ["Vendor"],
            "filament_type": ["PEKK-CF"],
            "nozzle_temperature": ["365"],
            "nozzle_temperature_initial_layer": ["365"]
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::write(
        fixture
            .bambu_source_root
            .join("Known PEKK-CF @Bambu Lab H2C 0.4 nozzle.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "type": "filament",
            "name": "Known PEKK-CF @Bambu Lab H2C 0.4 nozzle",
            "instantiation": "true",
            "from": "User",
            "filament_vendor": ["Known"],
            "filament_type": ["PEKK-CF"],
            "compatible_printers": ["Bambu Lab H2C 0.4 nozzle"],
            "filament_extruder_variant": [
                "Direct Drive Standard",
                "Direct Drive High Flow",
                "Direct Drive E3D High Flow"
            ],
            "nozzle_temperature": ["365", "365", "365"],
            "nozzle_temperature_initial_layer": ["365", "365", "365"]
        }))
        .unwrap(),
    )
    .unwrap();
    let sources = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec![
                "source:orca:system".to_owned(),
                "source:bambu:user".to_owned(),
            ],
        })
        .unwrap();
    let selected = sources
        .sources
        .iter()
        .find(|source| source.vendor == "Vendor")
        .unwrap();
    let dependency = sources
        .sources
        .iter()
        .find(|source| source.vendor == "Known")
        .unwrap();
    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();
    let mut request = BuildPlanRequest {
        source_catalog_id: sources.catalog_id,
        source_ids: vec![selected.id.0.clone()],
        target_catalog_id: targets.catalog_id,
        nozzles: vec![NozzleSelection {
            printer_id: "official:H2C".to_owned(),
            diameters: vec!["0.4".to_owned()],
        }],
        destination_account_id: "0000000000".to_owned(),
        preset_template: "{vendor} {material} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        outputs: OutputSelection::default(),
        naming: NamingOptions::default(),
    };
    let response = fixture.service.build_plan(request.clone()).unwrap();
    let value = serde_json::to_value(response).unwrap();

    assert_eq!(value["status"], "needs_resolution");
    assert_eq!(
        value["issues"][0]["source_candidates"][0]["source_id"],
        dependency.id.0
    );

    let source_decision = TargetTemplateDecision {
        issue_id: value["issues"][0]["id"].as_str().unwrap().to_owned(),
        action: TargetTemplateDecisionAction::UseSource,
        profile_name: None,
        source_id: Some(dependency.id.0.clone()),
    };
    let missing_inclusion = fixture
        .service
        .resolve_plan_dependencies(ResolvePlanDependenciesRequest {
            request: request.clone(),
            decisions: vec![source_decision.clone()],
        })
        .unwrap_err();
    assert!(
        missing_inclusion
            .to_string()
            .contains("must be included in the migration")
    );

    request.source_ids.push(dependency.id.0.clone());
    let plan = fixture
        .service
        .resolve_plan_dependencies(ResolvePlanDependenciesRequest {
            request,
            decisions: vec![source_decision],
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(plan.operations.len(), 2);
    assert!(
        plan.operations
            .iter()
            .all(|operation| operation.action.as_str() != "block"),
        "{:#?}",
        plan.operations
    );

    let dependency_path = fixture
        .bambu_source_root
        .join("Known PEKK-CF @Bambu Lab H2C 0.4 nozzle.json");
    let mut changed: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&dependency_path).unwrap()).unwrap();
    changed["nozzle_temperature"] = serde_json::json!(["366", "366", "366"]);
    std::fs::write(
        &dependency_path,
        serde_json::to_vec_pretty(&changed).unwrap(),
    )
    .unwrap();
    let before = std::fs::read(fixture.destination.join("filament/existing.json")).unwrap();
    let error = fixture
        .service
        .execute_plan_with(
            ExecutePlanRequest { plan_id: plan.id },
            &mut FakeProcess {
                checks: VecDeque::from([vec![]]),
            },
            &mut FakeClock::default(),
        )
        .unwrap_err();
    assert!(error.to_string().contains("fingerprint"), "{error}");
    assert_eq!(
        std::fs::read(fixture.destination.join("filament/existing.json")).unwrap(),
        before
    );
}

#[test]
fn build_plan_rejects_an_unsupported_nozzle_even_if_the_ui_is_bypassed() {
    let mut fixture = fixture_service(true);
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
            show_custom: true,
        })
        .unwrap();
    let custom = targets
        .printers
        .iter()
        .find(|printer| printer.id == "custom:Workshop CoreXY")
        .unwrap();
    assert!(!custom.nozzles[0].supported);

    let error = fixture
        .service
        .build_plan(BuildPlanRequest {
            source_catalog_id: sources.catalog_id,
            source_ids: vec![SOURCE_PROFILE_ID.to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: custom.id.clone(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
            naming: NamingOptions::default(),
        })
        .unwrap_err();

    assert!(error.to_string().contains("target nozzle is unsupported"));
}

#[test]
fn vendor_profiles_resolve_local_parents_before_filament_library() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("system/OrcaFilamentLibrary/filament");
    let vendor_root = temp.path().join("system/BBL/filament");
    std::fs::create_dir_all(&library_root).unwrap();
    std::fs::create_dir_all(&vendor_root).unwrap();
    std::fs::write(
        library_root.join("Shared @base.json"),
        br#"{"name":"Shared @base","filament_settings_id":["Shared @base"],"compatible_printers":["Library printer"]}"#,
    )
    .unwrap();
    std::fs::write(
        library_root.join("Library Only @base.json"),
        br#"{"name":"Library Only @base","filament_settings_id":["Library Only @base"],"compatible_printers":["Library fallback"]}"#,
    )
    .unwrap();
    std::fs::write(
        vendor_root.join("Shared @base.json"),
        br#"{"name":"Shared @base","filament_settings_id":["Shared @base"],"compatible_printers":["Vendor parent"]}"#,
    )
    .unwrap();
    std::fs::write(
        vendor_root.join("Shared @P1.json"),
        br#"{"name":"Shared @P1","inherits":"Shared @base","filament_settings_id":["Shared @P1"]}"#,
    )
    .unwrap();
    std::fs::write(
        vendor_root.join("Library Child @P1.json"),
        br#"{"name":"Library Child @P1","inherits":"Library Only @base","filament_settings_id":["Library Child @P1"]}"#,
    )
    .unwrap();
    let mut service = MigrationService::new(ServiceConfig {
        sources: vec![
            ApprovedSourceRoot {
                id: "source:orca:system:library".to_owned(),
                path: library_root,
                source_app: SourceApp::OrcaSlicer,
                source_kind: SourceKind::FactorySystem,
            },
            ApprovedSourceRoot {
                id: "source:orca:system:bbl".to_owned(),
                path: vendor_root,
                source_app: SourceApp::OrcaSlicer,
                source_kind: SourceKind::FactorySystem,
            },
        ],
        targets: Vec::new(),
        accounts: Vec::new(),
        data_root: temp.path().join("app-data"),
        process_close_timeout_ms: 200,
    })
    .unwrap();

    let catalog = service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec![
                "source:orca:system:library".to_owned(),
                "source:orca:system:bbl".to_owned(),
            ],
        })
        .unwrap();
    let shared_base = catalog
        .sources
        .iter()
        .find(|source| source.id.0 == "profile:orca:Shared @base")
        .unwrap();
    let shared_child = catalog
        .sources
        .iter()
        .find(|source| source.id.0 == "profile:orca:Shared @P1")
        .unwrap();
    let library_child = catalog
        .sources
        .iter()
        .find(|source| source.id.0 == "profile:orca:Library Child @P1")
        .unwrap();

    assert_eq!(
        shared_base.compatible_printers,
        ["Library printer".to_owned()].into_iter().collect()
    );
    assert_eq!(
        shared_child.compatible_printers,
        ["Vendor parent".to_owned()].into_iter().collect()
    );
    assert_eq!(
        library_child.compatible_printers,
        ["Library fallback".to_owned()].into_iter().collect()
    );
}

#[test]
fn signed_in_account_source_overrides_default_local_mirror() {
    let temp = tempfile::tempdir().unwrap();
    let default_root = temp.path().join("user/default/filament");
    let account_root = temp.path().join("user/2182110758/filament");
    std::fs::create_dir_all(&default_root).unwrap();
    std::fs::create_dir_all(&account_root).unwrap();
    std::fs::write(
        default_root.join("Mirror PLA.json"),
        br#"{"name":"Mirror PLA","filament_settings_id":["Mirror PLA"],"compatible_printers":["Bambu Lab X1 Carbon 0.4 nozzle"]}"#,
    )
    .unwrap();
    std::fs::write(
        account_root.join("Mirror PLA.json"),
        br#"{"name":"Mirror PLA","filament_settings_id":["Mirror PLA"],"compatible_printers":[]}"#,
    )
    .unwrap();
    let mut service = MigrationService::new(ServiceConfig {
        sources: vec![
            ApprovedSourceRoot {
                id: "source:orca:user:default".to_owned(),
                path: default_root,
                source_app: SourceApp::OrcaSlicer,
                source_kind: SourceKind::UserCustom,
            },
            ApprovedSourceRoot {
                id: "source:orca:user:2182110758".to_owned(),
                path: account_root,
                source_app: SourceApp::OrcaSlicer,
                source_kind: SourceKind::UserCustom,
            },
        ],
        targets: Vec::new(),
        accounts: Vec::new(),
        data_root: temp.path().join("app-data"),
        process_close_timeout_ms: 200,
    })
    .unwrap();

    let catalog = service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec![
                "source:orca:user:default".to_owned(),
                "source:orca:user:2182110758".to_owned(),
            ],
        })
        .unwrap();

    assert_eq!(catalog.sources.len(), 1);
    assert!(catalog.sources[0].compatible_printers.is_empty());
}

#[test]
#[ignore = "reads the local slicer installation without writing"]
fn installed_orca_sources_catalog_account_over_default_mirrors() {
    let (config, _) = ServiceConfig::current().unwrap();
    let root_ids = config
        .sources
        .iter()
        .filter(|root| root.source_app == SourceApp::OrcaSlicer)
        .map(|root| root.id.clone())
        .collect();
    let mut service = MigrationService::new(config).unwrap();

    let catalog = service
        .catalog_sources(CatalogSourcesRequest { root_ids })
        .unwrap();

    assert!(!catalog.sources.is_empty());
}

#[test]
#[ignore = "reads the local slicer installation without writing"]
fn installed_bambu_profiles_discover_and_catalog_h2c() {
    let (config, _) = ServiceConfig::current().unwrap();
    let mut service = MigrationService::new(config).unwrap();
    let discovery = service.discover();
    let target_id = discovery.target_catalog_ids.first().unwrap().clone();

    let targets = service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: target_id,
            show_custom: false,
        })
        .unwrap();

    let h2c = targets
        .printers
        .iter()
        .find(|printer| printer.code == "H2C")
        .unwrap();
    assert!(h2c.artwork_available);
    let artwork = service
        .printer_artwork(&targets.catalog_id, &h2c.id)
        .unwrap();
    assert!(artwork.starts_with(b"\x89PNG\r\n\x1a\n"));
}

#[test]
#[ignore = "reads installed profiles and proves every live profile file remains byte-identical"]
fn installed_elegoo_pet_cf_resolves_bambu_pet_cf_target_without_live_writes() {
    let (mut config, _) = ServiceConfig::current().unwrap();
    let app_data = tempfile::tempdir().unwrap();
    config.data_root = app_data.path().to_path_buf();
    let source_root_ids: Vec<_> = config
        .sources
        .iter()
        .filter(|root| root.source_app == SourceApp::OrcaSlicer)
        .map(|root| root.id.clone())
        .collect();
    let target_id = config.targets.first().unwrap().id.clone();
    let account_id = config
        .accounts
        .iter()
        .find(|account| account.account.eligibility.writable_by_default())
        .unwrap()
        .account
        .id
        .clone();
    let mut protected_roots: Vec<_> = config
        .sources
        .iter()
        .map(|root| root.path.clone())
        .collect();
    protected_roots.extend(
        config
            .targets
            .iter()
            .flat_map(|target| [target.profile_root.clone(), target.manifest_path.clone()]),
    );
    protected_roots.extend(
        config
            .accounts
            .iter()
            .map(|account| account.account.path.join("filament")),
    );
    let before = profile_tree_fingerprints(&protected_roots);
    let mut service = MigrationService::new(config).unwrap();

    let sources = service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: source_root_ids,
        })
        .unwrap();
    let elegoo = sources
        .sources
        .iter()
        .find(|source| source.vendor == "Elegoo" && source.material == "PET-CF")
        .unwrap();
    let targets = service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: target_id,
            show_custom: false,
        })
        .unwrap();
    let h2c = targets
        .printers
        .iter()
        .find(|printer| printer.code == "H2C")
        .unwrap();
    assert!(
        h2c.nozzles
            .iter()
            .any(|nozzle| nozzle.diameter == "0.4" && nozzle.supported)
    );
    let request = BuildPlanRequest {
        source_catalog_id: sources.catalog_id,
        source_ids: vec![elegoo.id.0.clone()],
        target_catalog_id: targets.catalog_id,
        nozzles: vec![NozzleSelection {
            printer_id: h2c.id.clone(),
            diameters: vec!["0.4".to_owned()],
        }],
        destination_account_id: account_id,
        preset_template: "{vendor} {material} {clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        outputs: OutputSelection::default(),
        naming: NamingOptions::default(),
    };
    let unresolved = service.build_plan(request.clone()).unwrap();
    let value = serde_json::to_value(unresolved).unwrap();
    assert_eq!(value["status"], "needs_resolution");
    let candidate = value["issues"][0]["installed_candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|candidate| {
            candidate["profile_name"]
                .as_str()
                .filter(|name| name.contains("Bambu PET-CF"))
        })
        .unwrap()
        .to_owned();
    let plan = service
        .resolve_plan_dependencies(ResolvePlanDependenciesRequest {
            request,
            decisions: vec![TargetTemplateDecision {
                issue_id: value["issues"][0]["id"].as_str().unwrap().to_owned(),
                action: TargetTemplateDecisionAction::UseInstalled,
                profile_name: Some(candidate),
                source_id: None,
            }],
        })
        .unwrap()
        .into_plan()
        .unwrap();

    assert_eq!(plan.operations.len(), 1);
    assert_ne!(plan.operations[0].action.as_str(), "block");
    assert_eq!(profile_tree_fingerprints(&protected_roots), before);
}

#[test]
fn manual_source_folders_are_validated_registered_and_removable() {
    let mut fixture = fixture_service(true);
    let manual = fixture._temp.path().join("manual-orca-filament");
    copy_tree(&fixture.source_root, &manual);
    let id = fixture
        .service
        .add_manual_source_folder(&manual, SourceApp::OrcaSlicer, SourceKind::FactorySystem)
        .unwrap();

    assert!(id.starts_with("source:manual:orca:"));
    assert!(fixture.service.discover().source_root_ids.contains(&id));
    assert!(
        fixture
            .service
            .data_root()
            .join("manual-source-roots-v1.json")
            .is_file()
    );
    let catalog = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec![id.clone()],
        })
        .unwrap();
    assert_eq!(catalog.sources.len(), 1);

    fixture.service.remove_manual_source_folder(&id).unwrap();
    assert!(!fixture.service.discover().source_root_ids.contains(&id));

    let empty = fixture._temp.path().join("empty-manual-source");
    std::fs::create_dir(&empty).unwrap();
    let error = fixture
        .service
        .add_manual_source_folder(&empty, SourceApp::OrcaSlicer, SourceKind::FactorySystem)
        .unwrap_err();
    assert!(error.to_string().contains("selectable filament profiles"));
}

#[test]
fn one_catalog_can_include_orca_and_bambu_sources_without_cross_app_inheritance() {
    let mut fixture = fixture_service(true);
    let bambu = fixture._temp.path().join("manual-bambu-filament");
    copy_tree(&fixtures().join("bambu-user-pre-sync"), &bambu);
    let original = bambu.join("Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json");
    let renamed = bambu.join("Alternate PLA Azure @Bambu Lab H2C 0.4 nozzle.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&original).unwrap()).unwrap();
    value["name"] = serde_json::json!("Alternate PLA Azure @Bambu Lab H2C 0.4 nozzle");
    value["filament_settings_id"] =
        serde_json::json!(["Alternate PLA Azure @Bambu Lab H2C 0.4 nozzle"]);
    value["filament_vendor"] = serde_json::json!(["Alternate"]);
    std::fs::write(&renamed, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    std::fs::remove_file(original).unwrap();
    fixture
        .service
        .add_manual_source_folder(&bambu, SourceApp::BambuStudio, SourceKind::UserCustom)
        .unwrap();

    let response = fixture
        .service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: fixture.service.discover().source_root_ids,
        })
        .unwrap();
    let apps: std::collections::BTreeSet<_> = response
        .sources
        .iter()
        .map(|source| source.source_app)
        .collect();
    assert_eq!(
        apps,
        std::collections::BTreeSet::from([SourceApp::OrcaSlicer, SourceApp::BambuStudio])
    );
    assert_eq!(
        response
            .sources
            .iter()
            .map(|source| &source.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        response.sources.len()
    );

    let targets = fixture
        .service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu".to_owned(),
            show_custom: false,
        })
        .unwrap();
    let plan = fixture
        .service
        .build_plan(BuildPlanRequest {
            source_catalog_id: response.catalog_id,
            source_ids: response
                .sources
                .iter()
                .map(|source| source.id.0.clone())
                .collect(),
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
            naming: NamingOptions::default(),
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(plan.operations.len(), 2);
    assert!(
        plan.operations
            .iter()
            .all(|operation| operation.action.as_str() == "create")
    );
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let result = fixture
        .service
        .execute_plan_with(
            ExecutePlanRequest { plan_id: plan.id },
            &mut process,
            &mut FakeClock::default(),
        )
        .unwrap();
    assert_eq!(result.committed_files, 6);
}

#[test]
fn command_requests_reject_arbitrary_paths_and_unapproved_root_ids() {
    let request = serde_json::from_value::<CatalogSourcesRequest>(serde_json::json!({
        "root_ids": ["source:orca:system"],
        "path": "C:/arbitrary"
    }));
    assert!(request.is_err());
    let manual = serde_json::from_value::<ManualSourceFolderRequest>(serde_json::json!({
        "source_app": "orca_slicer",
        "source_kind": "factory_system",
        "path": "C:/arbitrary"
    }));
    assert!(manual.is_err());

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
            source_ids: vec![SOURCE_PROFILE_ID.to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
            naming: NamingOptions {
                preset_rules: vec![],
                ams_rules: vec![rule],
                overrides: vec![NameOverride {
                    source_id: ProfileId::new(SOURCE_PROFILE_ID),
                    printer_id: "official:H2C".to_owned(),
                    nozzle: "0.4".to_owned(),
                    preset_name: Some("Aurora tuned H2C".to_owned()),
                    ams_name: None,
                }],
                conflict_decisions: vec![],
            },
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(response.operations[0].preset_name, "Aurora tuned H2C");
    assert_eq!(response.operations[0].ams_name, "Aurora");
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
            source_ids: vec![SOURCE_PROFILE_ID.to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
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
            source_ids: vec![SOURCE_PROFILE_ID.to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
            naming: NamingOptions::default(),
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(second.operations.len(), 1);
    assert_eq!(second.operations[0].action.as_str(), "skip");

    let mut no_op_process = FakeProcess {
        checks: VecDeque::from([vec![9], vec![9], vec![]]),
    };
    let mut no_op_clock = FakeClock::default();
    let result = fixture
        .service
        .execute_plan_with(
            ExecutePlanRequest { plan_id: second.id },
            &mut no_op_process,
            &mut no_op_clock,
        )
        .unwrap();
    assert_eq!(result.committed_files, 0);
    assert_eq!(result.created_files, 0);
    assert_eq!(result.updated_files, 0);
    assert_eq!(result.deleted_files, 0);
    assert_eq!(result.skipped_operations, 1);
    assert_eq!(no_op_clock.0, 0);
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
            source_ids: vec![SOURCE_PROFILE_ID.to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
            naming: NamingOptions::default(),
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(second.operations.len(), 1);
    assert_eq!(second.operations[0].action.as_str(), "block");
}

#[test]
fn per_run_output_controls_write_only_the_selected_artifact_kinds() {
    let mut slicing_only = fixture_service(true);
    let plan_id = build_plan_with_outputs(
        &mut slicing_only,
        OutputSelection {
            slicing_presets: true,
            custom_filaments: false,
        },
    );
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    let result = slicing_only
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap();
    assert_eq!(result.committed_files, 1);
    assert_eq!(result.created_files, 1);
    assert_eq!(result.updated_files, 0);
    assert_eq!(result.deleted_files, 0);
    assert_eq!(result.skipped_operations, 0);
    assert!(
        slicing_only
            .destination
            .join("filament/Aurora - H2C.json")
            .is_file()
    );
    assert!(
        !slicing_only
            .destination
            .join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json")
            .exists()
    );

    let mut custom_only = fixture_service(true);
    let plan_id = build_plan_with_outputs(
        &mut custom_only,
        OutputSelection {
            slicing_presets: false,
            custom_filaments: true,
        },
    );
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let result = custom_only
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap();
    assert_eq!(result.committed_files, 2);
    assert_eq!(result.created_files, 2);
    assert_eq!(result.updated_files, 0);
    assert_eq!(result.deleted_files, 0);
    assert_eq!(result.skipped_operations, 0);
    assert!(
        !custom_only
            .destination
            .join("filament/Aurora - H2C.json")
            .exists()
    );
    let custom = custom_only
        .destination
        .join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json");
    assert!(custom.is_file());
    assert!(custom.with_extension("info").is_file());
}

#[test]
fn slicing_only_ignores_unselected_custom_profile_conflicts() {
    let mut fixture = fixture_service(true);
    let plan_id = build_plan_with_outputs(
        &mut fixture,
        OutputSelection {
            slicing_presets: false,
            custom_filaments: true,
        },
    );
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap();

    let custom = fixture
        .destination
        .join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&custom).unwrap()).unwrap();
    value["filament_cost"] = serde_json::json!(["999"]);
    let changed_custom = serde_json::to_vec_pretty(&value).unwrap();
    std::fs::write(&custom, &changed_custom).unwrap();

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
            source_ids: vec![SOURCE_PROFILE_ID.to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection {
                slicing_presets: true,
                custom_filaments: false,
            },
            naming: NamingOptions::default(),
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(response.operations[0].action.as_str(), "create");

    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    fixture
        .service
        .execute_plan_with(
            ExecutePlanRequest {
                plan_id: response.id,
            },
            &mut process,
            &mut clock,
        )
        .unwrap();
    assert!(
        fixture
            .destination
            .join("filament/Aurora - H2C.json")
            .is_file()
    );
    assert_eq!(std::fs::read(custom).unwrap(), changed_custom);
}

#[test]
fn slicing_only_blocks_an_existing_normal_preset_with_different_settings() {
    let mut fixture = fixture_service(true);
    let destination = fixture.destination.join("filament/Aurora - H2C.json");
    let original = br#"{
        "name": "Aurora - H2C",
        "filament_settings_id": ["Aurora - H2C"],
        "filament_cost": ["999"],
        "compatible_printers": ["Bambu Lab H2C 0.4 nozzle"]
    }"#;
    std::fs::write(&destination, original).unwrap();

    let plan_id = build_plan_with_outputs(
        &mut fixture,
        OutputSelection {
            slicing_presets: true,
            custom_filaments: false,
        },
    );
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    let error = fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap_err();

    assert!(
        error.to_string().contains("unresolved blocking conflicts"),
        "{error}"
    );
    assert_eq!(std::fs::read(destination).unwrap(), original);
}

#[test]
fn local_execution_reports_each_real_transaction_phase() {
    let mut fixture = fixture_service(true);
    let plan_id = build_plan(&mut fixture);
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();
    let mut phases = Vec::new();

    fixture
        .service
        .execute_plan_with_progress(
            ExecutePlanRequest { plan_id },
            &mut process,
            &mut clock,
            |phase| phases.push(phase),
        )
        .unwrap();

    assert_eq!(
        phases,
        vec![
            ExecutionPhase::Validating,
            ExecutionPhase::ClosingBambu,
            ExecutionPhase::Staging,
            ExecutionPhase::ValidatingOutput,
            ExecutionPhase::BackingUp,
            ExecutionPhase::Committing,
            ExecutionPhase::WritingReceipt,
            ExecutionPhase::Finished,
        ]
    );
}

#[test]
fn local_result_exposes_receipt_backup_evidence() {
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
    let receipt = RunReceipt::load(&result.receipt_path).unwrap();

    assert_eq!(result.backup_sha256, receipt.backup.sha256);
    assert_eq!(result.backup_file_count, receipt.backup.file_count);
}

#[test]
fn add_target_preserves_existing_normal_preset_compatibility() {
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
    let response = fixture
        .service
        .build_plan(BuildPlanRequest {
            source_catalog_id: sources.catalog_id,
            source_ids: vec![SOURCE_PROFILE_ID.to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned(), "0.6".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
            naming: NamingOptions::default(),
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(
        response
            .operations
            .iter()
            .map(|operation| operation.action.as_str())
            .collect::<Vec<_>>(),
        vec!["skip", "add_target"]
    );
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    fixture
        .service
        .execute_plan_with(
            ExecutePlanRequest {
                plan_id: response.id,
            },
            &mut process,
            &mut clock,
        )
        .unwrap();

    let normal: serde_json::Value = serde_json::from_slice(
        &std::fs::read(fixture.destination.join("filament/Aurora - H2C.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        normal["compatible_printers"],
        serde_json::json!(["Bambu Lab H2C 0.4 nozzle", "Bambu Lab H2C 0.6 nozzle"])
    );
}

#[test]
fn destination_created_after_planning_is_not_silently_overwritten() {
    let mut fixture = fixture_service(true);
    let plan_id = build_plan(&mut fixture);
    let destination = fixture
        .destination
        .join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json");
    std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
    std::fs::write(&destination, b"appeared after planning").unwrap();
    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    let mut clock = FakeClock::default();

    let error = fixture
        .service
        .execute_plan_with(ExecutePlanRequest { plan_id }, &mut process, &mut clock)
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("destination precondition mismatch")
    );
    assert_eq!(
        std::fs::read(destination).unwrap(),
        b"appeared after planning"
    );
    let staging = fixture.service.data_root().join("staging");
    assert!(
        !staging.exists() || std::fs::read_dir(staging).unwrap().next().is_none(),
        "failed preflight left staged profile data behind"
    );
}

#[test]
fn explicit_update_preserves_cloud_sidecar_through_service_execution() {
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

    let profile_path = fixture
        .destination
        .join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json");
    let sidecar_path = profile_path.with_extension("info");
    std::fs::copy(
        fixtures().join("bambu-user-post-sync/custom.info"),
        &sidecar_path,
    )
    .unwrap();
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&profile_path).unwrap()).unwrap();
    value["filament_cost"] = serde_json::json!(["999"]);
    std::fs::write(&profile_path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

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
            source_ids: vec![SOURCE_PROFILE_ID.to_owned()],
            target_catalog_id: targets.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: "official:H2C".to_owned(),
                diameters: vec!["0.4".to_owned()],
            }],
            destination_account_id: "0000000000".to_owned(),
            preset_template: "{clean_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
            naming: NamingOptions {
                conflict_decisions: vec![ConflictDecision {
                    source_id: ProfileId::new(SOURCE_PROFILE_ID),
                    printer_id: "official:H2C".to_owned(),
                    nozzle: "0.4".to_owned(),
                    choice: ConflictChoice::Update,
                }],
                ..NamingOptions::default()
            },
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(response.operations[0].action.as_str(), "update");

    let mut process = FakeProcess {
        checks: VecDeque::from([vec![]]),
    };
    fixture
        .service
        .execute_plan_with(
            ExecutePlanRequest {
                plan_id: response.id,
            },
            &mut process,
            &mut clock,
        )
        .unwrap();

    let updated = InfoSidecar::parse(&std::fs::read(sidecar_path).unwrap()).unwrap();
    assert_eq!(updated.setting_id(), "PFUS0123456789abcd");
    assert_eq!(updated.sync_action(), SyncAction::Update);
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(profile_path).unwrap()).unwrap();
    assert_eq!(value["filament_cost"], serde_json::json!(["24.99"]));
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
