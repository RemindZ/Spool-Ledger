use bambu_filament_migrator::profiles::{InfoSidecar, SyncAction};
use std::path::PathBuf;

fn fixture(path: &str) -> Vec<u8> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures");
    std::fs::read(root.join(path)).unwrap()
}

#[test]
fn pre_sync_sidecar_round_trips_byte_for_byte() {
    let bytes = fixture("synthetic/bambu-user-pre-sync/custom.info");
    let parsed = InfoSidecar::parse(&bytes).unwrap();
    assert_eq!(parsed.to_bytes(), bytes);
    assert!(parsed.setting_id().is_empty());
    assert_eq!(parsed.sync_action(), SyncAction::None);
}

#[test]
fn post_sync_sidecar_exposes_cloud_id_without_ams_claim() {
    let bytes = fixture("synthetic/bambu-user-post-sync/custom.info");
    let parsed = InfoSidecar::parse(&bytes).unwrap();
    assert_eq!(parsed.setting_id(), "PFUS0123456789abcd");
    assert_eq!(parsed.to_bytes(), bytes);
}

#[test]
fn sync_actions_are_characterized() {
    for (value, expected) in [
        ("", SyncAction::None),
        ("create", SyncAction::Create),
        ("update", SyncAction::Update),
        ("delete", SyncAction::Delete),
        ("hold", SyncAction::Hold),
    ] {
        let bytes = format!(
            "sync_info = {value}\nuser_id = \nsetting_id = \nbase_id = \nupdated_time = 1787140000\n"
        );
        let parsed = InfoSidecar::parse(bytes.as_bytes()).unwrap();
        assert_eq!(parsed.sync_action(), expected);
        assert_eq!(parsed.to_bytes(), bytes.as_bytes());
    }
}

#[test]
fn malformed_sidecar_is_rejected() {
    let error = InfoSidecar::parse(b"setting_id = PFUSabc\n").unwrap_err();
    assert!(error.to_string().contains("missing sidecar field"));
}
