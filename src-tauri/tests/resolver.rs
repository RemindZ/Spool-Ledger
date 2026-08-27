use bambu_filament_migrator::field_policy::{FieldClass, FieldPolicyTable};
use bambu_filament_migrator::model::SourceApp;
use bambu_filament_migrator::resolver::{CatalogRoot, ProfileCatalog};
use std::path::PathBuf;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic")
}

fn orca_catalog() -> ProfileCatalog {
    ProfileCatalog::load_roots(&[
        CatalogRoot::system(fixtures().join("orca-system"), SourceApp::OrcaSlicer),
        CatalogRoot::user(fixtures().join("orca-user"), SourceApp::OrcaSlicer),
    ])
    .unwrap()
}

#[test]
fn child_override_wins_and_keeps_provenance() {
    let catalog = orca_catalog();
    let resolved = catalog.resolve_name("Northstar PLA Aurora - H2C").unwrap();
    assert_eq!(resolved.string_values("filament_flow_ratio"), vec!["0.95"]);
    assert_eq!(resolved.string_values("slow_down_layer_time"), vec!["15"]);
    assert_eq!(
        resolved
            .provenance("filament_flow_ratio")
            .unwrap()
            .file_name(),
        "Northstar PLA Aurora @base.json"
    );
    assert_eq!(
        resolved
            .provenance("slow_down_layer_time")
            .unwrap()
            .file_name(),
        "Northstar PLA Aurora @BBL X1C.json"
    );
}

#[test]
fn abstract_bases_are_not_selectable() {
    let catalog = orca_catalog();
    let names: Vec<_> = catalog
        .selectable_profiles()
        .iter()
        .map(|profile| profile.name.as_str())
        .collect();
    assert!(names.contains(&"Northstar PLA Aurora @BBL X1C"));
    assert!(!names.contains(&"Northstar PLA Aurora @base"));
}

#[test]
fn missing_parent_is_blocking() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Broken.json"),
        r#"{"name":"Broken","inherits":"Absent","instantiation":"true"}"#,
    )
    .unwrap();
    let catalog =
        ProfileCatalog::load_roots(&[CatalogRoot::system(dir.path(), SourceApp::OrcaSlicer)])
            .unwrap();
    let error = catalog.resolve_name("Broken").unwrap_err();
    assert!(error.to_string().contains("missing parent"));
}

#[test]
fn inheritance_cycle_is_blocking() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("A.json"), r#"{"name":"A","inherits":"B"}"#).unwrap();
    std::fs::write(dir.path().join("B.json"), r#"{"name":"B","inherits":"A"}"#).unwrap();
    let catalog =
        ProfileCatalog::load_roots(&[CatalogRoot::system(dir.path(), SourceApp::OrcaSlicer)])
            .unwrap();
    let error = catalog.resolve_name("A").unwrap_err();
    assert!(error.to_string().contains("cycle"));
}

#[test]
fn unrelated_equal_priority_roots_remain_ambiguous() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    std::fs::write(
        first.path().join("Duplicate.json"),
        r#"{"name":"Duplicate","filament_settings_id":["Duplicate"]}"#,
    )
    .unwrap();
    std::fs::write(
        second.path().join("Duplicate.json"),
        r#"{"name":"Duplicate","filament_settings_id":["Duplicate"]}"#,
    )
    .unwrap();

    let error = ProfileCatalog::load_roots(&[
        CatalogRoot::user(first.path(), SourceApp::OrcaSlicer),
        CatalogRoot::user(second.path(), SourceApp::OrcaSlicer),
    ])
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("ambiguous profile name Duplicate")
    );
}

#[test]
fn catalog_loading_reports_each_profile_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("One.json"),
        r#"{"name":"One","filament_settings_id":["One"]}"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("Two.json"),
        r#"{"name":"Two","filament_settings_id":["Two"]}"#,
    )
    .unwrap();
    let mut loaded = 0;

    ProfileCatalog::load_roots_with_progress(
        &[CatalogRoot::system(dir.path(), SourceApp::OrcaSlicer)],
        || loaded += 1,
    )
    .unwrap();

    assert_eq!(loaded, 2);
}

#[test]
fn current_orca_material_fields_have_explicit_transfer_policies() {
    let policy = FieldPolicyTable::bundled().unwrap();
    for field in [
        "complete_print_exhaust_fan_speed",
        "during_print_exhaust_fan_speed",
        "filament_is_support",
        "filament_minimal_purge_on_wipe_tower",
        "filament_scarf_gap",
        "filament_scarf_height",
        "filament_scarf_length",
        "filament_scarf_seam_type",
        "filament_shrink",
        "filament_soluble",
        "overhang_fan_threshold",
        "reduce_fan_stop_start_freq",
    ] {
        assert_eq!(
            policy.classify(field),
            Some(FieldClass::SourceMaterial),
            "{field}"
        );
    }
    for field in [
        "filament_deretraction_speed",
        "filament_long_retractions_when_cut",
        "filament_retract_before_wipe",
        "filament_retract_restart_extra",
        "filament_retract_when_changing_layer",
        "filament_retraction_distances_when_cut",
        "filament_retraction_length",
        "filament_retraction_minimum_travel",
        "filament_retraction_speed",
        "filament_wipe",
        "filament_wipe_distance",
        "filament_z_hop",
        "filament_z_hop_types",
    ] {
        assert_eq!(policy.classify(field), Some(FieldClass::Mapped), "{field}");
    }
}

#[test]
fn current_bambu_custom_profile_fields_have_explicit_policies() {
    let policy = FieldPolicyTable::bundled().unwrap();
    for field in [
        "enable_overhang_bridge_fan",
        "enable_pressure_advance",
        "overhang_threshold_participating_cooling",
    ] {
        assert_eq!(
            policy.classify(field),
            Some(FieldClass::SourceMaterial),
            "{field}"
        );
    }
    assert_eq!(
        policy.classify("default_filament_colour"),
        Some(FieldClass::Metadata)
    );
}

#[test]
fn reported_fan_fields_are_characterized_as_source_material() {
    let policy = FieldPolicyTable::bundled().unwrap();

    for field in ["first_x_layer_part_fan_speed", "ironing_fan_speed"] {
        assert_eq!(
            policy.classify(field),
            Some(FieldClass::SourceMaterial),
            "{field}"
        );
    }
}

#[test]
fn current_bambu_h2c_fields_have_explicit_target_policies() {
    let policy = FieldPolicyTable::bundled().unwrap();
    for field in [
        "additional_fan_full_speed_layer",
        "circle_compensation_speed",
        "close_additional_fan_first_x_layers",
        "cooling_perimeter_transition_distance",
        "cooling_slowdown_logic",
        "counter_coef_1",
        "counter_coef_2",
        "counter_coef_3",
        "counter_limit_max",
        "counter_limit_min",
        "diameter_limit",
        "filament_adaptive_volumetric_speed",
        "filament_adhesiveness_category",
        "filament_bridge_speed",
        "filament_change_length",
        "filament_change_length_nc",
        "filament_contact_safe",
        "filament_cooling_before_tower",
        "filament_dev_drying_cooling_temperature",
        "filament_dev_drying_softening_temperature",
        "filament_emission_safe",
        "filament_enable_overhang_speed",
        "filament_extruder_compatibility",
        "filament_flush_temp",
        "filament_flush_temp_fast",
        "filament_flush_volumetric_speed",
        "filament_ingredients_safe",
        "filament_long_retractions_when_ec",
        "filament_metal_stickiness",
        "filament_overhang_1_4_speed",
        "filament_overhang_2_4_speed",
        "filament_overhang_3_4_speed",
        "filament_overhang_4_4_speed",
        "filament_overhang_totally_speed",
        "filament_pre_cooling_temperature",
        "filament_pre_cooling_temperature_nc",
        "filament_preheat_temperature_delta",
        "filament_prime_volume",
        "filament_prime_volume_nc",
        "filament_printable",
        "filament_ramming_travel_time",
        "filament_ramming_travel_time_nc",
        "filament_ramming_volumetric_speed",
        "filament_ramming_volumetric_speed_nc",
        "filament_retract_length_nc",
        "filament_retraction_distances_when_ec",
        "filament_tower_interface_pre_extrusion_dist",
        "filament_tower_interface_pre_extrusion_length",
        "filament_tower_interface_print_temp",
        "filament_tower_interface_purge_volume",
        "filament_tower_ironing_area",
        "filament_velocity_adaptation_factor",
        "hole_coef_1",
        "hole_coef_2",
        "hole_coef_3",
        "hole_limit_max",
        "hole_limit_min",
        "impact_strength_z",
        "long_retractions_when_ec",
        "no_slow_down_for_cooling_on_outwalls",
        "override_process_overhang_speed",
        "pre_start_fan_time",
        "retraction_distances_when_ec",
        "volumetric_speed_coefficients",
    ] {
        assert_eq!(
            policy.classify(field),
            Some(FieldClass::TargetMachine),
            "{field}"
        );
    }
    assert_eq!(policy.classify("description"), Some(FieldClass::Metadata));
}

#[test]
fn field_policy_blocks_unknown_cross_application_fields() {
    let policy = FieldPolicyTable::bundled().unwrap();
    assert_eq!(
        policy.classify("filament_flow_ratio"),
        Some(FieldClass::Mapped)
    );
    assert_eq!(
        policy.classify("filament_start_gcode"),
        Some(FieldClass::TargetMachine)
    );
    assert_eq!(
        policy.classify("compatible_printers"),
        Some(FieldClass::Derived)
    );
    let error = policy
        .validate_cross_application(["filament_flow_ratio", "future_unknown_key"])
        .unwrap_err();
    assert!(error.to_string().contains("future_unknown_key"));
}
