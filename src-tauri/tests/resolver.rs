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
