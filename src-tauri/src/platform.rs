use crate::AppError;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformDefaults {
    pub bambu_config: PathBuf,
    pub orca_config: PathBuf,
    pub bambu_executables: Vec<PathBuf>,
    pub orca_executables: Vec<PathBuf>,
}

impl PlatformDefaults {
    pub fn current() -> Option<Self> {
        let base = BaseDirs::new()?;
        let config = base.config_dir();
        let mut bambu_executables = Vec::new();
        let mut orca_executables = Vec::new();
        #[cfg(target_os = "windows")]
        {
            bambu_executables.push(PathBuf::from(
                r"C:\Program Files\Bambu Studio\bambu-studio.exe",
            ));
            orca_executables.push(PathBuf::from(
                r"C:\Program Files\OrcaSlicer\orca-slicer.exe",
            ));
        }
        #[cfg(target_os = "macos")]
        {
            bambu_executables.push(PathBuf::from(
                "/Applications/BambuStudio.app/Contents/MacOS/BambuStudio",
            ));
            orca_executables.push(PathBuf::from(
                "/Applications/OrcaSlicer.app/Contents/MacOS/OrcaSlicer",
            ));
        }
        #[cfg(target_os = "linux")]
        {
            bambu_executables.push(PathBuf::from("/usr/bin/bambu-studio"));
            orca_executables.push(PathBuf::from("/usr/bin/orca-slicer"));
        }
        Some(Self {
            bambu_config: config.join("BambuStudio"),
            orca_config: config.join("OrcaSlicer"),
            bambu_executables,
            orca_executables,
        })
    }
}

pub fn ensure_within(root: &Path, candidate: &Path) -> Result<PathBuf, AppError> {
    let root = root
        .canonicalize()
        .map_err(|error| AppError::io(root, error))?;
    let candidate = candidate
        .canonicalize()
        .map_err(|error| AppError::io(candidate, error))?;
    if candidate.starts_with(&root) {
        Ok(candidate)
    } else {
        Err(AppError::UnsafePath(candidate))
    }
}

pub fn candidate_executable(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find(|path| path.is_file()).cloned()
}
