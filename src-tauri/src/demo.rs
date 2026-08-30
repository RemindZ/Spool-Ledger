use crate::commands::{ApprovedAccount, ApprovedSourceRoot, ApprovedTargetCatalog, ServiceConfig};
use crate::discovery::inspect_account;
use crate::error::AppError;
use crate::model::{SourceApp, SourceKind};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct DemoWorkspace {
    root: PathBuf,
}

impl DemoWorkspace {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn cleanup(&self) -> Result<(), AppError> {
        match std::fs::remove_dir_all(&self.root) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(AppError::io(&self.root, error)),
        }
    }
}

impl Drop for DemoWorkspace {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

pub fn create_demo_service_config() -> Result<(ServiceConfig, DemoWorkspace), AppError> {
    let root = std::env::temp_dir().join(format!("spool-ledger-demo-{}", Uuid::new_v4().simple()));
    std::fs::create_dir_all(&root).map_err(|error| AppError::io(&root, error))?;
    let workspace = DemoWorkspace { root: root.clone() };

    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic");
    let source_root = root.join("orca-filament");
    let target_root = root.join("bambu-system");
    copy_tree(&fixtures.join("orca-system/filament"), &source_root)?;
    copy_tree(&fixtures.join("bambu-system"), &target_root)?;

    let destination = root.join("account/demo-user");
    let destination_filament = destination.join("filament");
    std::fs::create_dir_all(&destination_filament)
        .map_err(|error| AppError::io(&destination_filament, error))?;
    let existing = destination_filament.join("existing.json");
    std::fs::write(
        &existing,
        br#"{"name":"Existing Demo PLA","filament_settings_id":["Existing Demo PLA"]}"#,
    )
    .map_err(|error| AppError::io(&existing, error))?;

    let executable = root.join("BambuStudio-demo.exe");
    std::fs::write(&executable, b"synthetic demo executable")
        .map_err(|error| AppError::io(&executable, error))?;

    let config = ServiceConfig {
        sources: vec![ApprovedSourceRoot {
            id: "source:demo:orca:system".to_owned(),
            path: source_root,
            source_app: SourceApp::OrcaSlicer,
            source_kind: SourceKind::FactorySystem,
        }],
        targets: vec![ApprovedTargetCatalog {
            id: "target:demo:bambu".to_owned(),
            manifest_path: target_root.join("BBL.json"),
            profile_root: target_root.join("filament"),
            custom_machine_root: Some(target_root.join("machine")),
        }],
        accounts: vec![ApprovedAccount {
            account: inspect_account(&destination)?,
            bambu_executable: Some(executable),
        }],
        data_root: root.join("app-data"),
        process_close_timeout_ms: 200,
    };

    Ok((config, workspace))
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), AppError> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.map_err(|error| {
            AppError::InvalidProfile(format!("synthetic demo fixture is unreadable: {error}"))
        })?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target).map_err(|error| AppError::io(&target, error))?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|error| AppError::io(parent, error))?;
            }
            std::fs::copy(entry.path(), &target).map_err(|error| AppError::io(&target, error))?;
        }
    }
    Ok(())
}

pub struct DemoProcessBackend;

impl crate::sync::ProcessBackend for DemoProcessBackend {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AccountEligibility;

    #[test]
    fn demo_config_keeps_every_approved_path_inside_one_temporary_workspace() {
        let (config, workspace) = create_demo_service_config().unwrap();
        let root = workspace.root().to_path_buf();

        assert!(root.starts_with(std::env::temp_dir()));
        assert!(config.data_root.starts_with(&root));
        assert!(
            config
                .sources
                .iter()
                .all(|source| source.path.starts_with(&root))
        );
        assert!(config.targets.iter().all(|target| {
            target.manifest_path.starts_with(&root)
                && target.profile_root.starts_with(&root)
                && target
                    .custom_machine_root
                    .as_ref()
                    .is_none_or(|path| path.starts_with(&root))
        }));
        assert!(config.accounts.iter().all(|account| {
            account.account.path.starts_with(&root)
                && account
                    .bambu_executable
                    .as_ref()
                    .is_none_or(|path| path.starts_with(&root))
        }));
        assert_eq!(
            config.accounts[0].account.eligibility,
            AccountEligibility::Eligible
        );
    }

    #[test]
    fn demo_workspace_removes_its_temporary_tree_when_dropped() {
        let root = {
            let (_, workspace) = create_demo_service_config().unwrap();
            assert!(workspace.root().is_dir());
            workspace.root().to_path_buf()
        };

        assert!(!root.exists());
    }

    #[test]
    fn demo_workspace_can_be_cleaned_before_tauri_exits_the_process() {
        let (_, workspace) = create_demo_service_config().unwrap();
        let root = workspace.root().to_path_buf();

        workspace.cleanup().unwrap();

        assert!(!root.exists());
    }

    #[test]
    fn demo_process_backend_never_observes_or_launches_real_processes() {
        use crate::sync::ProcessBackend;

        let mut backend = super::DemoProcessBackend;
        assert!(backend.bambu_processes().unwrap().is_empty());
        assert!(backend.request_graceful_close(&[42]).unwrap());
        backend.launch(Path::new("synthetic-demo.exe")).unwrap();
    }
}
