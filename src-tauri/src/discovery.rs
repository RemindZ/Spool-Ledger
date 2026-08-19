use crate::AppError;
use crate::model::AccountEligibility;
use crate::platform::{PlatformDefaults, candidate_executable};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountRoot {
    pub id: String,
    pub path: PathBuf,
    pub eligibility: AccountEligibility,
    pub filament_profile_count: usize,
}

impl AccountRoot {
    pub fn writable_by_default(&self) -> bool {
        self.eligibility.writable_by_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installation {
    pub name: String,
    pub config_root: PathBuf,
    pub executable: Option<PathBuf>,
    pub detected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoverySnapshot {
    pub bambu: Installation,
    pub orca: Installation,
    pub bambu_accounts: Vec<AccountRoot>,
    pub orca_accounts: Vec<AccountRoot>,
}

#[derive(Debug, Clone)]
pub struct DiscoveryService {
    defaults: PlatformDefaults,
}

impl DiscoveryService {
    pub fn new(defaults: PlatformDefaults) -> Self {
        Self { defaults }
    }

    pub fn current() -> Result<Self, AppError> {
        PlatformDefaults::current()
            .map(Self::new)
            .ok_or_else(|| AppError::InvalidProfile("home/config directory unavailable".to_owned()))
    }

    pub fn scan(&self) -> Result<DiscoverySnapshot, AppError> {
        Ok(DiscoverySnapshot {
            bambu: Installation {
                name: "Bambu Studio".to_owned(),
                detected: self.defaults.bambu_config.is_dir(),
                executable: candidate_executable(&self.defaults.bambu_executables),
                config_root: self.defaults.bambu_config.clone(),
            },
            orca: Installation {
                name: "OrcaSlicer".to_owned(),
                detected: self.defaults.orca_config.is_dir(),
                executable: candidate_executable(&self.defaults.orca_executables),
                config_root: self.defaults.orca_config.clone(),
            },
            bambu_accounts: discover_accounts(&self.defaults.bambu_config)?,
            orca_accounts: discover_accounts(&self.defaults.orca_config)?,
        })
    }
}

pub fn inspect_account(path: impl AsRef<Path>) -> Result<AccountRoot, AppError> {
    let path = path.as_ref().to_path_buf();
    let id = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_owned();
    let lower = id.to_ascii_lowercase();
    let filament = path.join("filament");
    let filament_profile_count = if filament.is_dir() {
        WalkDir::new(&filament)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
            })
            .count()
    } else {
        0
    };
    let eligibility = if lower.contains("backup") {
        AccountEligibility::Backup
    } else if matches!(lower.as_str(), "default" | "local" | "default_user") {
        AccountEligibility::DefaultLocal
    } else if filament_profile_count > 0 {
        AccountEligibility::Eligible
    } else if path
        .read_dir()
        .map_or(true, |mut entries| entries.next().is_none())
    {
        AccountEligibility::Empty
    } else if filament.is_dir() {
        AccountEligibility::Stale
    } else {
        AccountEligibility::Unsupported
    };
    Ok(AccountRoot {
        id,
        path,
        eligibility,
        filament_profile_count,
    })
}

fn discover_accounts(config_root: &Path) -> Result<Vec<AccountRoot>, AppError> {
    let user_root = config_root.join("user");
    if !user_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut accounts = Vec::new();
    for entry in std::fs::read_dir(&user_root).map_err(|error| AppError::io(&user_root, error))? {
        let entry = entry.map_err(|error| AppError::io(&user_root, error))?;
        if entry
            .file_type()
            .map_err(|error| AppError::io(entry.path(), error))?
            .is_dir()
        {
            accounts.push(inspect_account(entry.path())?);
        }
    }
    accounts.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(accounts)
}
