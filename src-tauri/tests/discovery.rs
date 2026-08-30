use bambu_filament_migrator::discovery::{AccountRoot, inspect_account};
use bambu_filament_migrator::model::{AccountEligibility, PrinterPresetKind};
#[cfg(target_os = "macos")]
use bambu_filament_migrator::platform::PlatformDefaults;
use bambu_filament_migrator::platform::ensure_within;
use bambu_filament_migrator::targets::TargetCatalog;
use std::path::PathBuf;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic")
}

#[cfg(target_os = "macos")]
#[test]
fn macos_defaults_target_application_bundles_and_support_roots() {
    let defaults = PlatformDefaults::current().unwrap();
    assert!(
        defaults
            .bambu_config
            .ends_with("Library/Application Support/BambuStudio")
    );
    assert!(
        defaults
            .orca_config
            .ends_with("Library/Application Support/OrcaSlicer")
    );
    assert_eq!(
        defaults.bambu_executables,
        [PathBuf::from(
            "/Applications/BambuStudio.app/Contents/MacOS/BambuStudio",
        )]
    );
    assert_eq!(
        defaults.orca_executables,
        [PathBuf::from(
            "/Applications/OrcaSlicer.app/Contents/MacOS/OrcaSlicer",
        )]
    );
}

#[test]
fn empty_account_is_not_writable_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let account = inspect_account(dir.path()).unwrap();
    assert_eq!(account.eligibility, AccountEligibility::Empty);
    assert!(!account.writable_by_default());
}

#[test]
fn populated_account_is_eligible() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("filament")).unwrap();
    std::fs::write(
        dir.path().join("filament/User PLA.json"),
        r#"{"name":"User PLA","filament_settings_id":["User PLA"]}"#,
    )
    .unwrap();
    let account: AccountRoot = inspect_account(dir.path()).unwrap();
    assert_eq!(account.eligibility, AccountEligibility::Eligible);
    assert!(account.writable_by_default());
}

#[test]
fn official_targets_and_nozzles_come_from_manifest_profiles() {
    let catalog = TargetCatalog::load(&fixtures().join("bambu-system/BBL.json"), None).unwrap();
    let h2c = catalog
        .printers()
        .iter()
        .find(|item| item.code == "H2C")
        .unwrap();
    assert_eq!(h2c.kind, PrinterPresetKind::Official);
    assert!(h2c.verified);
    let nozzles: Vec<_> = h2c
        .nozzles
        .iter()
        .map(|item| item.diameter.as_str())
        .collect();
    assert_eq!(nozzles, vec!["0.2", "0.4", "0.6", "0.8"]);
}

#[test]
fn official_targets_load_from_installed_vendor_subdirectory_layout() {
    let profiles = tempfile::tempdir().unwrap();
    let vendor = profiles.path().join("BBL");
    std::fs::create_dir_all(vendor.join("machine")).unwrap();
    std::fs::create_dir_all(vendor.join("filament")).unwrap();
    std::fs::write(
        profiles.path().join("BBL.json"),
        r#"{"machine_model_list":[{"name":"Bambu Lab Demo","sub_path":"machine/Demo.json"}]}"#,
    )
    .unwrap();
    std::fs::write(
        vendor.join("machine/Demo.json"),
        r#"{"name":"Bambu Lab Demo","nozzle_diameter":[]}"#,
    )
    .unwrap();
    std::fs::write(
        vendor.join("filament/Generic PLA @BBL Demo.json"),
        r#"{"compatible_printers":["Bambu Lab Demo 0.4 nozzle"],"filament_extruder_variant":["Direct Drive High Flow"]}"#,
    )
    .unwrap();

    let catalog = TargetCatalog::load(&profiles.path().join("BBL.json"), None).unwrap();
    let printer = catalog.printers().first().unwrap();
    assert_eq!(printer.nozzles[0].diameter, "0.4");
    assert_eq!(printer.extruder_variants, vec!["Direct Drive High Flow"]);
}

#[test]
fn target_catalog_reports_each_filament_metadata_file_once() {
    let root = fixtures().join("bambu-system");
    let expected = walkdir::WalkDir::new(root.join("filament"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
        })
        .count();
    let mut processed = 0;

    TargetCatalog::load_with_progress(&root.join("BBL.json"), None, || processed += 1).unwrap();

    assert_eq!(processed, expected);
}

#[test]
fn custom_printers_are_hidden_without_toggle() {
    let custom = tempfile::tempdir().unwrap();
    std::fs::write(
        custom.path().join("Workshop CoreXY.json"),
        r#"{"name":"Workshop CoreXY 0.4 nozzle","nozzle_diameter":["0.4"]}"#,
    )
    .unwrap();
    let catalog = TargetCatalog::load(
        &fixtures().join("bambu-system/BBL.json"),
        Some(custom.path()),
    )
    .unwrap();
    assert_eq!(catalog.visible(false).len(), 1);
    assert_eq!(catalog.visible(true).len(), 2);
    assert_eq!(catalog.visible(true)[1].kind, PrinterPresetKind::Custom);
    assert!(!catalog.visible(true)[1].verified);
}

#[test]
fn path_guard_rejects_escape() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let error = ensure_within(root.path(), outside.path()).unwrap_err();
    assert!(error.to_string().contains("unsafe path"));
}
