use bambu_filament_migrator::commands::{
    ApprovedAccount, ApprovedSourceRoot, ApprovedTargetCatalog, BuildPlanRequest,
    CatalogSourcesRequest, CatalogTargetsRequest, ExecutePlanRequest, MigrationService,
    NozzleSelection, OutputSelection, ServiceConfig,
};
use bambu_filament_migrator::discovery::inspect_account;
use bambu_filament_migrator::model::{SourceApp, SourceKind};
use bambu_filament_migrator::planner::{NamingOptions, PlanAction};
use bambu_filament_migrator::profiles::InfoSidecar;
use bambu_filament_migrator::receipt::RunReceipt;
use bambu_filament_migrator::sync::{Clock, ProcessBackend};
use bambu_filament_migrator::transaction::Transaction;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const SYNTHETIC_ACCOUNT_ID: &str = "0000000000";

#[derive(Deserialize)]
struct CopiedRootsManifest {
    live_roots: BTreeMap<String, PathBuf>,
    copied_roots: BTreeMap<String, PathBuf>,
    hashes: BTreeMap<String, BTreeMap<String, String>>,
}

fn characterization_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(".local-characterization")
}

fn hash_file(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn hash_tree(root: &Path) -> BTreeMap<String, String> {
    let mut hashes = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.unwrap();
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        hashes.insert(relative, hash_file(entry.path()));
    }
    hashes
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in walkdir::WalkDir::new(source).follow_links(false) {
        let entry = entry.unwrap();
        let target = destination.join(entry.path().strip_prefix(source).unwrap());
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(target).unwrap();
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn remove_panchroma_outputs(account: &Path) {
    let filament = account.join("filament");
    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(&filament).follow_links(false) {
        let entry = entry.unwrap();
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("json")
        {
            continue;
        }
        let value: Value = serde_json::from_slice(&std::fs::read(entry.path()).unwrap()).unwrap();
        if value
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name.contains("Panchroma"))
        {
            paths.push(entry.path().to_path_buf());
            paths.push(entry.path().with_extension("info"));
        }
    }
    for path in paths {
        assert!(path.starts_with(&filament));
        if path.is_file() {
            std::fs::remove_file(path).unwrap();
        }
    }
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

#[derive(Default)]
struct StoppedProcess;

impl ProcessBackend for StoppedProcess {
    fn bambu_processes(&mut self) -> Result<Vec<u32>, String> {
        Ok(Vec::new())
    }

    fn request_graceful_close(&mut self, _process_ids: &[u32]) -> Result<bool, String> {
        Ok(true)
    }

    fn launch(&mut self, _executable: &Path) -> Result<(), String> {
        Ok(())
    }
}

#[test]
#[ignore = "requires ignored copies of installed OrcaSlicer and Bambu Studio roots"]
fn copied_windows_roots_reproduce_all_panchroma_h2c_artifacts_without_live_writes() {
    let characterization = characterization_root();
    let manifest: CopiedRootsManifest = serde_json::from_slice(
        &std::fs::read(characterization.join("copied-roots-manifest.json")).unwrap(),
    )
    .unwrap();
    let live_before: BTreeMap<_, _> = manifest
        .live_roots
        .iter()
        .map(|(name, root)| (name.clone(), hash_tree(root)))
        .collect();
    assert_eq!(live_before, manifest.hashes);
    let live_bambu_manifest = manifest.live_roots["bambu_system"]
        .parent()
        .unwrap()
        .join("BBL.json");
    let copied_bambu_manifest = manifest.copied_roots["bambu_system"].join("BBL.json");
    let bambu_manifest_hash = hash_file(&live_bambu_manifest);
    assert_eq!(hash_file(&copied_bambu_manifest), bambu_manifest_hash);
    for (name, copied) in &manifest.copied_roots {
        let mut copied_hashes = hash_tree(copied);
        if name == "bambu_system" {
            copied_hashes.remove("BBL.json");
        }
        assert_eq!(copied_hashes, manifest.hashes[name]);
    }

    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("account").join(SYNTHETIC_ACCOUNT_ID);
    copy_tree(&manifest.copied_roots["bambu_account"], &destination);
    remove_panchroma_outputs(&destination);
    let destination_filament_file_count_before = hash_tree(&destination.join("filament")).len();
    let executable = temp.path().join("BambuStudio.exe");
    std::fs::write(&executable, b"characterization stub").unwrap();
    let account = inspect_account(&destination).unwrap();
    assert!(account.writable_by_default());

    let mut service = MigrationService::new(ServiceConfig {
        sources: vec![ApprovedSourceRoot {
            id: "source:orca:copied".to_owned(),
            path: manifest.copied_roots["orca"].clone(),
            source_app: SourceApp::OrcaSlicer,
            source_kind: SourceKind::FactorySystem,
        }],
        targets: vec![ApprovedTargetCatalog {
            id: "target:bambu:copied".to_owned(),
            manifest_path: manifest.copied_roots["bambu_system"].join("BBL.json"),
            profile_root: manifest.copied_roots["bambu_system"].join("filament"),
            custom_machine_root: None,
        }],
        accounts: vec![ApprovedAccount {
            account,
            bambu_executable: Some(executable),
        }],
        data_root: temp.path().join("app-data"),
        process_close_timeout_ms: 200,
    })
    .unwrap();
    let source_catalog = service
        .catalog_sources(CatalogSourcesRequest {
            root_ids: vec!["source:orca:copied".to_owned()],
        })
        .unwrap();
    let source_ids: Vec<_> = source_catalog
        .sources
        .iter()
        .filter(|source| source.name.starts_with("Panchroma"))
        .map(|source| source.id.0.clone())
        .collect();
    assert_eq!(source_ids.len(), 16);

    let target_catalog = service
        .catalog_targets(CatalogTargetsRequest {
            catalog_id: "target:bambu:copied".to_owned(),
            show_custom: false,
        })
        .unwrap();
    let h2c = target_catalog
        .printers
        .iter()
        .find(|printer| printer.code == "H2C")
        .unwrap();
    let nozzles: BTreeSet<_> = h2c
        .nozzles
        .iter()
        .filter(|nozzle| nozzle.supported)
        .map(|nozzle| nozzle.diameter.clone())
        .collect();
    assert_eq!(
        nozzles,
        BTreeSet::from([
            "0.2".to_owned(),
            "0.4".to_owned(),
            "0.6".to_owned(),
            "0.8".to_owned()
        ])
    );

    let response = service
        .build_plan(BuildPlanRequest {
            source_catalog_id: source_catalog.catalog_id,
            source_ids,
            target_catalog_id: target_catalog.catalog_id,
            nozzles: vec![NozzleSelection {
                printer_id: h2c.id.clone(),
                diameters: nozzles.into_iter().collect(),
            }],
            destination_account_id: SYNTHETIC_ACCOUNT_ID.to_owned(),
            preset_template: "{source_name} - {printer_code}".to_owned(),
            ams_template: "{vendor} {material} {clean_name}".to_owned(),
            outputs: OutputSelection::default(),
            naming: NamingOptions::default(),
        })
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(response.operations.len(), 64);
    assert!(
        response
            .operations
            .iter()
            .all(|operation| operation.action == PlanAction::Create)
    );

    let mut process = StoppedProcess;
    let mut clock = FakeClock::default();
    let local = service
        .execute_plan_with(
            ExecutePlanRequest {
                plan_id: response.id,
            },
            &mut process,
            &mut clock,
        )
        .unwrap();
    assert_eq!(local.committed_files, 144);

    let mut normal_count = 0;
    let mut custom_count = 0;
    let mut sidecar_count = 0;
    let mut ids = BTreeMap::<String, usize>::new();
    for entry in walkdir::WalkDir::new(destination.join("filament")).follow_links(false) {
        let entry = entry.unwrap();
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(destination.join("filament"))
            .unwrap();
        if entry.path().extension().and_then(|value| value.to_str()) == Some("info") {
            if entry.file_name().to_string_lossy().contains("Panchroma") {
                let sidecar = InfoSidecar::parse(&std::fs::read(entry.path()).unwrap()).unwrap();
                assert!(sidecar.setting_id().is_empty());
                sidecar_count += 1;
            }
            continue;
        }
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let value: Value = serde_json::from_slice(&std::fs::read(entry.path()).unwrap()).unwrap();
        let name = value.get("name").and_then(Value::as_str).unwrap_or("");
        if !name.contains("Panchroma") {
            continue;
        }
        if relative.parent() == Some(Path::new("base")) {
            custom_count += 1;
            assert_eq!(value.get("inherits"), Some(&Value::String(String::new())));
            assert!(value.get("setting_id").is_none());
            *ids.entry(
                value
                    .get("filament_id")
                    .and_then(Value::as_str)
                    .unwrap()
                    .to_owned(),
            )
            .or_default() += 1;
        } else {
            normal_count += 1;
        }
    }
    assert_eq!((normal_count, custom_count, sidecar_count), (16, 64, 64));
    assert_eq!(ids.len(), 16);
    assert!(ids.values().all(|count| *count == 4));

    let run_root = service.data_root().join("runs").join(&local.run_id);
    let transaction = Transaction::load(&run_root).unwrap();
    assert_eq!(transaction.journal().entries.len(), 144);
    assert!(transaction.journal().entries.iter().all(|entry| {
        entry
            .destination_path
            .starts_with(destination.canonicalize().unwrap())
    }));
    let receipt = RunReceipt::load(&local.receipt_path).unwrap();
    assert_eq!(receipt.counts.committed, 144);
    assert_eq!(
        receipt.backup.file_count,
        destination_filament_file_count_before
    );

    let live_after: BTreeMap<_, _> = manifest
        .live_roots
        .iter()
        .map(|(name, root)| (name.clone(), hash_tree(root)))
        .collect();
    assert_eq!(live_after, live_before);
    assert_eq!(hash_file(&live_bambu_manifest), bambu_manifest_hash);
}
