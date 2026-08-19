use bambu_filament_migrator::discovery::{AccountRoot, inspect_account};
use bambu_filament_migrator::model::{AccountEligibility, PrinterPresetKind};
use bambu_filament_migrator::platform::ensure_within;
use bambu_filament_migrator::targets::TargetCatalog;
use std::path::PathBuf;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic")
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
