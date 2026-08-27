use bambu_filament_migrator::model::{ProfileId, SourceApp, SourceKind};
use bambu_filament_migrator::planner::{
    DestinationIndex, MigrationRequest, MigrationSource, MigrationStatus, PlanAction, Planner,
    TargetSelection,
};
use bambu_filament_migrator::profiles::InfoSidecar;
use bambu_filament_migrator::resolver::{CatalogRoot, ProfileCatalog};
use bambu_filament_migrator::writer::{
    BambuAdapter, GeneratedArtifactKind, TargetProfileReference, Writer, WriterContext,
    effective_settings_fingerprint, material_settings_fingerprint,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic")
}

fn catalogs() -> (ProfileCatalog, ProfileCatalog) {
    let root = fixtures();
    let source = ProfileCatalog::load_roots(&[CatalogRoot::system(
        root.join("orca-system/filament"),
        SourceApp::OrcaSlicer,
    )])
    .unwrap();
    let target = ProfileCatalog::load_roots(&[CatalogRoot::system(
        root.join("bambu-system/filament"),
        SourceApp::BambuStudio,
    )])
    .unwrap();
    (source, target)
}

fn request(source_catalog: &ProfileCatalog, nozzles: &[&str]) -> MigrationRequest {
    let source_id = "Northstar PLA Aurora @BBL X1C";
    let effective = source_catalog.resolve_name(source_id).unwrap();
    MigrationRequest {
        sources: vec![MigrationSource {
            id: ProfileId::new(source_id),
            name: "Northstar PLA Aurora".to_owned(),
            vendor: "Northstar".to_owned(),
            material: "PLA".to_owned(),
            family: "Northstar PLA".to_owned(),
            variant: "Aurora".to_owned(),
            source_app: SourceApp::OrcaSlicer,
            source_kind: SourceKind::FactorySystem,
            compatible_printers: BTreeSet::from(["Bambu Lab X1 Carbon".to_owned()]),
            migration_status: MigrationStatus::New,
            source_precondition_fingerprint: effective_settings_fingerprint(&effective).unwrap(),
            existing_filament_id: None,
        }],
        targets: nozzles
            .iter()
            .map(|nozzle| TargetSelection {
                printer_id: "official:H2C".to_owned(),
                printer_name: "Bambu Lab H2C".to_owned(),
                printer_code: "H2C".to_owned(),
                nozzle: (*nozzle).to_owned(),
                printer_preset_name: format!("Bambu Lab H2C {nozzle} nozzle"),
                custom_unverified: false,
            })
            .collect(),
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    }
}

fn context_for<'a>(
    source_catalog: &'a ProfileCatalog,
    target_catalog: &'a ProfileCatalog,
    source_name: &str,
) -> WriterContext<'a> {
    WriterContext {
        sources: BTreeMap::from([(
            ProfileId::new(source_name),
            source_catalog.resolve_name(source_name).unwrap(),
        )]),
        targets: target_catalog,
        target_profiles: BTreeMap::from([(
            "official:H2C".to_owned(),
            TargetProfileReference::Catalog("Generic PLA @BBL H2C".to_owned()),
        )]),
        adapter: BambuAdapter::v2_0_0_56(),
        outputs: bambu_filament_migrator::writer::OutputSelection::default(),
        updated_time: 1_787_140_000,
        existing_sidecars: BTreeMap::new(),
        existing_normal_compatibility: BTreeMap::new(),
    }
}

fn context<'a>(
    source_catalog: &'a ProfileCatalog,
    target_catalog: &'a ProfileCatalog,
) -> WriterContext<'a> {
    context_for(
        source_catalog,
        target_catalog,
        "Northstar PLA Aurora @BBL X1C",
    )
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn adapter_owns_the_expected_cloud_setting_id_prefix() {
    assert_eq!(BambuAdapter::v2_0_0_56().setting_id_prefix, "PFUS");
}

#[test]
fn stages_normal_and_flattened_profiles_for_selected_nozzles_only() {
    let (sources, targets) = catalogs();
    let plan = Planner::build(
        &request(&sources, &["0.6", "0.4"]),
        &DestinationIndex::default(),
    )
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let stage = temp.path().join("stage");
    let staged = Writer::stage(&plan, &context(&sources, &targets), &stage).unwrap();

    assert_eq!(staged.artifacts.len(), 5);
    assert_eq!(
        staged
            .artifacts
            .iter()
            .filter(|item| item.kind == GeneratedArtifactKind::NormalPreset)
            .count(),
        1
    );
    assert!(staged.artifacts.iter().all(|item| {
        !item.relative_path.to_string_lossy().contains("0.2 nozzle")
            && !item.relative_path.to_string_lossy().contains("0.8 nozzle")
    }));

    let normal = read_json(&stage.join("filament/Aurora - H2C.json"));
    assert_eq!(normal["inherits"], "Generic PLA @BBL H2C");
    assert_eq!(
        normal["compatible_printers"],
        serde_json::json!(["Bambu Lab H2C 0.4 nozzle", "Bambu Lab H2C 0.6 nozzle"])
    );
    assert_eq!(
        normal["filament_flow_ratio"],
        serde_json::json!(["0.95", "0.95", "0.99"])
    );

    let custom_path =
        stage.join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json");
    let custom = read_json(&custom_path);
    assert_eq!(custom["inherits"], "");
    assert_eq!(custom["from"], "User");
    assert_eq!(
        custom["compatible_printers"],
        serde_json::json!(["Bambu Lab H2C 0.4 nozzle"])
    );
    assert_eq!(
        custom["filament_extruder_variant"],
        serde_json::json!([
            "Direct Drive Standard",
            "Direct Drive High Flow",
            "Direct Drive E3D High Flow"
        ])
    );
    assert_eq!(
        custom["filament_max_volumetric_speed"],
        serde_json::json!(["16", "16", "12"])
    );
    assert_eq!(
        custom["nozzle_temperature"],
        serde_json::json!(["230", "230", "220"])
    );
    assert_eq!(
        custom["filament_retraction_length"],
        serde_json::json!(["0.4", "0.4", "0.4"])
    );
    assert_eq!(custom["filament_wipe"], serde_json::json!(["0", "0", "1"]));
    assert_eq!(
        custom["filament_start_gcode"],
        serde_json::json!(["; synthetic target gcode\n"])
    );
    assert_eq!(custom["filament_cost"], serde_json::json!(["24.99"]));
    assert_eq!(custom["filament_id"], plan.operations[0].filament_id);
    assert!(custom.get("setting_id").is_none());
    assert!(custom.get("include").is_none());
    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/expected/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json");
    assert_eq!(custom, read_json(&golden));
}

#[test]
fn target_owned_fields_do_not_change_material_equivalence() {
    let (sources, targets) = catalogs();
    let plan = Planner::build(&request(&sources, &["0.4"]), &DestinationIndex::default()).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let stage = temp.path().join("stage");
    let writer_context = context(&sources, &targets);
    Writer::stage(&plan, &writer_context, &stage).unwrap();
    let mut destination =
        read_json(&stage.join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.json"));
    let baseline = material_settings_fingerprint(&destination, &writer_context.adapter).unwrap();

    destination["filament_start_gcode"] = serde_json::json!(["; changed target gcode\n"]);
    destination["name"] = serde_json::json!("Renamed destination");

    assert_eq!(
        material_settings_fingerprint(&destination, &writer_context.adapter).unwrap(),
        baseline
    );
}

#[test]
fn writer_never_invents_cloud_setting_id_and_sidecar_is_exact() {
    let (sources, targets) = catalogs();
    let plan = Planner::build(&request(&sources, &["0.4"]), &DestinationIndex::default()).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let stage = temp.path().join("stage");
    Writer::stage(&plan, &context(&sources, &targets), &stage).unwrap();
    let bytes = std::fs::read(
        stage.join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.info"),
    )
    .unwrap();
    assert_eq!(
        bytes,
        b"sync_info = \nuser_id = \nsetting_id = \nbase_id = \nupdated_time = 1787140000\n"
    );
    let sidecar = InfoSidecar::parse(&bytes).unwrap();
    assert_eq!(sidecar.setting_id(), "");
    assert_eq!(sidecar.user_id(), "");
    assert!(!String::from_utf8(bytes).unwrap().contains("PFUS"));
}

#[test]
fn update_preserves_existing_cloud_identity_in_sidecar() {
    let (sources, targets) = catalogs();
    let mut plan =
        Planner::build(&request(&sources, &["0.4"]), &DestinationIndex::default()).unwrap();
    plan.operations[0].action = PlanAction::Update;
    let operation_id = plan.operations[0].id.clone();
    let existing = InfoSidecar::parse(
        &std::fs::read(fixtures().join("bambu-user-post-sync/custom.info")).unwrap(),
    )
    .unwrap();
    let mut writer_context = context(&sources, &targets);
    writer_context
        .existing_sidecars
        .insert(operation_id, existing);
    let temp = tempfile::tempdir().unwrap();
    let stage = temp.path().join("stage");
    Writer::stage(&plan, &writer_context, &stage).unwrap();
    let updated = InfoSidecar::parse(
        &std::fs::read(
            stage.join("filament/base/Northstar PLA Aurora @Bambu Lab H2C 0.4 nozzle.info"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        updated.sync_action(),
        bambu_filament_migrator::profiles::SyncAction::Update
    );
    assert_eq!(updated.setting_id(), "PFUS0123456789abcd");
    assert_eq!(updated.updated_time(), 1_787_140_000);
}

#[test]
fn source_fingerprint_mismatch_blocks_staging_before_any_files_are_written() {
    let (sources, targets) = catalogs();
    let mut request = request(&sources, &["0.4"]);
    request.sources[0].source_precondition_fingerprint = "changed-after-plan".to_owned();
    let plan = Planner::build(&request, &DestinationIndex::default()).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let stage = temp.path().join("stage");
    let error = Writer::stage(&plan, &context(&sources, &targets), &stage).unwrap_err();
    assert!(error.to_string().contains("fingerprint"));
    assert!(!stage.exists());
}

#[test]
fn unclassified_cross_application_field_blocks_staging() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("Unknown PLA.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "type": "filament",
            "name": "Unknown PLA",
            "instantiation": "true",
            "filament_vendor": ["Synthetic"],
            "filament_type": ["PLA"],
            "nozzle_temperature": ["220"],
            "unreviewed_orca_knob": ["1"]
        }))
        .unwrap(),
    )
    .unwrap();
    let sources =
        ProfileCatalog::load_roots(&[CatalogRoot::system(root.path(), SourceApp::OrcaSlicer)])
            .unwrap();
    let (_, targets) = catalogs();
    let effective = sources.resolve_name("Unknown PLA").unwrap();
    let request = MigrationRequest {
        sources: vec![MigrationSource {
            id: ProfileId::new("Unknown PLA"),
            name: "Unknown PLA".to_owned(),
            vendor: "Synthetic".to_owned(),
            material: "PLA".to_owned(),
            family: "Unknown".to_owned(),
            variant: String::new(),
            source_app: SourceApp::OrcaSlicer,
            source_kind: SourceKind::FactorySystem,
            compatible_printers: BTreeSet::new(),
            migration_status: MigrationStatus::Unsupported,
            source_precondition_fingerprint: effective_settings_fingerprint(&effective).unwrap(),
            existing_filament_id: None,
        }],
        targets: vec![TargetSelection {
            printer_id: "official:H2C".to_owned(),
            printer_name: "Bambu Lab H2C".to_owned(),
            printer_code: "H2C".to_owned(),
            nozzle: "0.4".to_owned(),
            printer_preset_name: "Bambu Lab H2C 0.4 nozzle".to_owned(),
            custom_unverified: false,
        }],
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    };
    let plan = Planner::build(&request, &DestinationIndex::default()).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let stage = temp.path().join("stage");
    let error = Writer::stage(
        &plan,
        &context_for(&sources, &targets, "Unknown PLA"),
        &stage,
    )
    .unwrap_err();
    assert!(error.to_string().contains("unreviewed_orca_knob"));
    assert!(!stage.exists());
}

#[test]
fn characterized_fan_fields_survive_cross_application_transfer() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("Cooling PLA.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "type": "filament",
            "name": "Cooling PLA",
            "instantiation": "true",
            "filament_vendor": ["Synthetic"],
            "filament_type": ["PLA"],
            "nozzle_temperature": ["220"],
            "first_x_layer_part_fan_speed": ["41"],
            "ironing_fan_speed": ["62"]
        }))
        .unwrap(),
    )
    .unwrap();
    let sources =
        ProfileCatalog::load_roots(&[CatalogRoot::system(root.path(), SourceApp::OrcaSlicer)])
            .unwrap();
    let (_, targets) = catalogs();
    let effective = sources.resolve_name("Cooling PLA").unwrap();
    let migration = MigrationRequest {
        sources: vec![MigrationSource {
            id: ProfileId::new("Cooling PLA"),
            name: "Cooling PLA".to_owned(),
            vendor: "Synthetic".to_owned(),
            material: "PLA".to_owned(),
            family: "Cooling".to_owned(),
            variant: String::new(),
            source_app: SourceApp::OrcaSlicer,
            source_kind: SourceKind::FactorySystem,
            compatible_printers: BTreeSet::new(),
            migration_status: MigrationStatus::New,
            source_precondition_fingerprint: effective_settings_fingerprint(&effective).unwrap(),
            existing_filament_id: None,
        }],
        targets: vec![TargetSelection {
            printer_id: "official:H2C".to_owned(),
            printer_name: "Bambu Lab H2C".to_owned(),
            printer_code: "H2C".to_owned(),
            nozzle: "0.4".to_owned(),
            printer_preset_name: "Bambu Lab H2C 0.4 nozzle".to_owned(),
            custom_unverified: false,
        }],
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    };
    let plan = Planner::build(&migration, &DestinationIndex::default()).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let stage = temp.path().join("stage");

    Writer::stage(
        &plan,
        &context_for(&sources, &targets, "Cooling PLA"),
        &stage,
    )
    .unwrap();

    let custom = read_json(
        &stage.join("filament/base/Synthetic PLA Cooling @Bambu Lab H2C 0.4 nozzle.json"),
    );
    assert_eq!(
        custom["first_x_layer_part_fan_speed"],
        serde_json::json!(["41"])
    );
    assert_eq!(custom["ironing_fan_speed"], serde_json::json!(["62"]));
}

#[test]
fn generated_path_components_cannot_escape_the_staging_tree() {
    let (sources, targets) = catalogs();
    let mut plan =
        Planner::build(&request(&sources, &["0.4"]), &DestinationIndex::default()).unwrap();
    plan.operations[0].printer_preset_name = "../escape".to_owned();
    let temp = tempfile::tempdir().unwrap();
    let stage = temp.path().join("stage");
    let error = Writer::stage(&plan, &context(&sources, &targets), &stage).unwrap_err();
    assert!(error.to_string().contains("unsafe"));
    assert!(!temp.path().join("escape.json").exists());
    assert!(!stage.exists());
}

#[test]
fn refuses_to_stage_over_existing_files() {
    let (sources, targets) = catalogs();
    let plan = Planner::build(&request(&sources, &["0.4"]), &DestinationIndex::default()).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let stage = temp.path().join("stage");
    std::fs::create_dir(&stage).unwrap();
    std::fs::write(stage.join("unrelated.txt"), b"keep").unwrap();
    let error = Writer::stage(&plan, &context(&sources, &targets), &stage).unwrap_err();
    assert!(error.to_string().contains("staging"));
    assert_eq!(std::fs::read(stage.join("unrelated.txt")).unwrap(), b"keep");
}
