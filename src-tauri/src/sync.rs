use crate::AppError;
use crate::model::{EvidenceLevel, OperationState};
use crate::profiles::InfoSidecar;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use sysinfo::{ProcessesToUpdate, System};

pub trait Clock {
    fn now_ms(&self) -> u64;
    fn sleep_ms(&mut self, duration: u64);
}

pub trait ProcessBackend {
    fn bambu_processes(&mut self) -> Result<Vec<u32>, String>;
    fn request_graceful_close(&mut self, process_ids: &[u32]) -> Result<bool, String>;
    fn launch(&mut self, executable: &Path) -> Result<(), String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseOutcome {
    AlreadyStopped,
    Stopped,
}

pub struct ProcessController;

impl ProcessController {
    pub fn ensure_stopped(
        backend: &mut impl ProcessBackend,
        clock: &mut impl Clock,
        timeout_ms: u64,
        poll_interval_ms: u64,
    ) -> Result<CloseOutcome, AppError> {
        let process_ids = backend
            .bambu_processes()
            .map_err(AppError::InvalidProfile)?;
        if process_ids.is_empty() {
            return Ok(CloseOutcome::AlreadyStopped);
        }
        let accepted = backend
            .request_graceful_close(&process_ids)
            .map_err(AppError::InvalidProfile)?;
        if !accepted {
            return Err(AppError::BambuStillRunning);
        }
        let started = clock.now_ms();
        loop {
            if clock.now_ms().saturating_sub(started) >= timeout_ms {
                return Err(AppError::BambuStillRunning);
            }
            if backend
                .bambu_processes()
                .map_err(AppError::InvalidProfile)?
                .is_empty()
            {
                return Ok(CloseOutcome::Stopped);
            }
            clock.sleep_ms(poll_interval_ms.max(1));
        }
    }

    pub fn launch(backend: &mut impl ProcessBackend, executable: &Path) -> Result<(), AppError> {
        backend.launch(executable).map_err(|error| {
            AppError::InvalidProfile(format!(
                "failed to launch {}: {error}",
                executable.display()
            ))
        })
    }
}

pub struct SystemClock {
    started: Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    fn sleep_ms(&mut self, duration: u64) {
        std::thread::sleep(Duration::from_millis(duration));
    }
}

pub struct SystemProcessBackend {
    system: System,
}

impl Default for SystemProcessBackend {
    fn default() -> Self {
        Self {
            system: System::new_all(),
        }
    }
}

impl ProcessBackend for SystemProcessBackend {
    fn bambu_processes(&mut self) -> Result<Vec<u32>, String> {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        let mut result: Vec<_> = self
            .system
            .processes()
            .iter()
            .filter(|(_, process)| is_bambu_process(process))
            .map(|(pid, _)| pid.as_u32())
            .collect();
        result.sort_unstable();
        Ok(result)
    }

    fn request_graceful_close(&mut self, process_ids: &[u32]) -> Result<bool, String> {
        request_platform_close(process_ids)
    }

    fn launch(&mut self, executable: &Path) -> Result<(), String> {
        Command::new(executable)
            .spawn()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

fn is_bambu_process(process: &sysinfo::Process) -> bool {
    let name = process.name().to_string_lossy().to_ascii_lowercase();
    let executable = process
        .exe()
        .and_then(Path::file_stem)
        .map(|value| value.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    [name.as_str(), executable.as_str()].iter().any(|value| {
        matches!(
            *value,
            "bambu-studio" | "bambu-studio.exe" | "bambustudio" | "bambustudio.exe"
        )
    })
}

#[cfg(target_os = "windows")]
fn request_platform_close(process_ids: &[u32]) -> Result<bool, String> {
    use std::collections::BTreeSet;
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
    };

    struct Context {
        process_ids: BTreeSet<u32>,
        sent: bool,
    }

    unsafe extern "system" fn callback(window: HWND, parameter: LPARAM) -> i32 {
        let context = unsafe { &mut *(parameter as *mut Context) };
        let mut process_id = 0_u32;
        unsafe { GetWindowThreadProcessId(window, &mut process_id) };
        if context.process_ids.contains(&process_id)
            && unsafe { PostMessageW(window, WM_CLOSE, 0, 0) } != 0
        {
            context.sent = true;
        }
        1
    }

    let mut context = Context {
        process_ids: process_ids.iter().copied().collect(),
        sent: false,
    };
    let result = unsafe { EnumWindows(Some(callback), &mut context as *mut Context as LPARAM) };
    if result == 0 {
        Err(std::io::Error::last_os_error().to_string())
    } else {
        Ok(context.sent)
    }
}

#[cfg(target_os = "macos")]
fn request_platform_close(_process_ids: &[u32]) -> Result<bool, String> {
    let status = Command::new("osascript")
        .args(["-e", "tell application \"BambuStudio\" to quit"])
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(true)
    } else {
        Err(format!("osascript exited with {status}"))
    }
}

#[cfg(target_os = "linux")]
fn request_platform_close(_process_ids: &[u32]) -> Result<bool, String> {
    Ok(false)
}

pub trait SyncRuntime {
    fn read(&mut self, path: &Path) -> Result<Option<Vec<u8>>, String>;
    fn now_ms(&self) -> u64;
    fn sleep_ms(&mut self, duration: u64);
    fn is_cancelled(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncExpectation {
    pub operation_id: String,
    pub info_path: PathBuf,
    pub initial_sha256: String,
    pub setting_id_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncObservation {
    pub operation_id: String,
    pub info_path: PathBuf,
    pub evidence: EvidenceLevel,
    pub state: OperationState,
    pub setting_id: Option<String>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncResult {
    pub timed_out: bool,
    pub highest_evidence: EvidenceLevel,
    pub observations: Vec<SyncObservation>,
}

pub struct SyncMonitor;

impl SyncMonitor {
    pub fn wait(
        expectations: &[SyncExpectation],
        runtime: &mut impl SyncRuntime,
        timeout_ms: u64,
        poll_interval_ms: u64,
    ) -> Result<SyncResult, AppError> {
        let started = runtime.now_ms();
        let mut observations: Vec<_> = expectations
            .iter()
            .map(|expectation| SyncObservation {
                operation_id: expectation.operation_id.clone(),
                info_path: expectation.info_path.clone(),
                evidence: EvidenceLevel::CreatedLocal,
                state: OperationState::CreatedLocalUnsynchronized,
                setting_id: None,
                diagnostic: None,
            })
            .collect();
        loop {
            if runtime.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            let mut candidates: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for (index, expectation) in expectations.iter().enumerate() {
                match runtime
                    .read(&expectation.info_path)
                    .map_err(AppError::InvalidProfile)?
                {
                    Some(bytes) => {
                        let changed = sha256_bytes(&bytes) != expectation.initial_sha256;
                        if changed {
                            observations[index].evidence = EvidenceLevel::LoadedByBambu;
                            observations[index].state = OperationState::LoadedByBambu;
                            observations[index].diagnostic = None;
                        }
                        match InfoSidecar::parse(&bytes) {
                            Ok(sidecar) => {
                                let setting_id = sidecar.setting_id();
                                if !setting_id.is_empty() {
                                    if setting_id.starts_with(&expectation.setting_id_prefix) {
                                        candidates
                                            .entry(setting_id.to_owned())
                                            .or_default()
                                            .push(index);
                                    } else if changed {
                                        observations[index].diagnostic = Some(format!(
                                            "setting id has unexpected prefix: {setting_id}"
                                        ));
                                    }
                                }
                            }
                            Err(error) if changed => {
                                observations[index].diagnostic = Some(error.to_string());
                            }
                            Err(_) => {}
                        }
                    }
                    None => {
                        observations[index].diagnostic =
                            Some("expected sidecar is missing".to_owned());
                    }
                }
            }
            for (setting_id, indices) in candidates {
                if indices.len() == 1 {
                    let observation = &mut observations[indices[0]];
                    observation.evidence = EvidenceLevel::CloudIdAssigned;
                    observation.state = OperationState::CloudIdAssigned;
                    observation.setting_id = Some(setting_id);
                    observation.diagnostic = None;
                } else {
                    for index in indices {
                        observations[index].diagnostic =
                            Some(format!("duplicate cloud setting id: {setting_id}"));
                    }
                }
            }
            if !observations.is_empty()
                && observations
                    .iter()
                    .all(|item| item.evidence == EvidenceLevel::CloudIdAssigned)
            {
                return Ok(sync_result(false, observations));
            }
            if runtime.now_ms().saturating_sub(started) >= timeout_ms {
                return Ok(sync_result(true, observations));
            }
            runtime.sleep_ms(poll_interval_ms.max(1));
        }
    }
}

fn sync_result(timed_out: bool, observations: Vec<SyncObservation>) -> SyncResult {
    let highest_evidence = observations
        .iter()
        .map(|item| item.evidence)
        .max_by_key(|evidence| evidence_rank(*evidence))
        .unwrap_or(EvidenceLevel::CreatedLocal);
    SyncResult {
        timed_out,
        highest_evidence,
        observations,
    }
}

fn evidence_rank(evidence: EvidenceLevel) -> u8 {
    match evidence {
        EvidenceLevel::CreatedLocal => 0,
        EvidenceLevel::LoadedByBambu => 1,
        EvidenceLevel::CloudIdAssigned => 2,
        EvidenceLevel::AmsVerified => 3,
    }
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub struct FsSyncRuntime {
    started: Instant,
    cancelled: Arc<AtomicBool>,
}

impl FsSyncRuntime {
    pub fn new(cancelled: Arc<AtomicBool>) -> Self {
        Self {
            started: Instant::now(),
            cancelled,
        }
    }
}

impl SyncRuntime for FsSyncRuntime {
    fn read(&mut self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    fn now_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    fn sleep_ms(&mut self, duration: u64) {
        std::thread::sleep(Duration::from_millis(duration));
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}
