use bambu_filament_migrator::model::{EvidenceLevel, OperationState};
use bambu_filament_migrator::sync::{
    Clock, CloseOutcome, ProcessBackend, ProcessController, SyncExpectation, SyncMonitor,
    SyncRuntime, sha256_bytes,
};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Default)]
struct FakeClock {
    now: u64,
}

impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.now
    }

    fn sleep_ms(&mut self, duration: u64) {
        self.now += duration;
    }
}

struct FakeProcessBackend {
    checks: VecDeque<Vec<u32>>,
    close_requests: Vec<Vec<u32>>,
    close_accepted: bool,
    launch_error: Option<String>,
}

impl ProcessBackend for FakeProcessBackend {
    fn bambu_processes(&mut self) -> Result<Vec<u32>, String> {
        Ok(self.checks.pop_front().unwrap_or_default())
    }

    fn request_graceful_close(&mut self, process_ids: &[u32]) -> Result<bool, String> {
        self.close_requests.push(process_ids.to_vec());
        Ok(self.close_accepted)
    }

    fn launch(&mut self, _executable: &Path) -> Result<(), String> {
        self.launch_error.clone().map_or(Ok(()), Err)
    }
}

#[test]
fn process_controller_handles_already_stopped_and_graceful_exit() {
    let mut stopped = FakeProcessBackend {
        checks: VecDeque::from([vec![]]),
        close_requests: Vec::new(),
        close_accepted: true,
        launch_error: None,
    };
    let mut clock = FakeClock::default();
    assert_eq!(
        ProcessController::ensure_stopped(&mut stopped, &mut clock, 1_000, 100).unwrap(),
        CloseOutcome::AlreadyStopped
    );
    assert!(stopped.close_requests.is_empty());

    let mut running = FakeProcessBackend {
        checks: VecDeque::from([vec![42], vec![42], vec![]]),
        close_requests: Vec::new(),
        close_accepted: true,
        launch_error: None,
    };
    assert_eq!(
        ProcessController::ensure_stopped(&mut running, &mut clock, 1_000, 100).unwrap(),
        CloseOutcome::Stopped
    );
    assert_eq!(running.close_requests, vec![vec![42]]);
}

#[test]
fn process_controller_never_forces_a_process_that_remains_open() {
    let mut backend = FakeProcessBackend {
        checks: VecDeque::from([vec![42], vec![42], vec![42], vec![42]]),
        close_requests: Vec::new(),
        close_accepted: true,
        launch_error: None,
    };
    let mut clock = FakeClock::default();
    let error = ProcessController::ensure_stopped(&mut backend, &mut clock, 200, 100).unwrap_err();
    assert!(error.to_string().contains("still running"));
    assert_eq!(backend.close_requests.len(), 1);
}

#[test]
fn launch_failure_preserves_the_exact_executable_error() {
    let mut backend = FakeProcessBackend {
        checks: VecDeque::new(),
        close_requests: Vec::new(),
        close_accepted: true,
        launch_error: Some("executable was removed".to_owned()),
    };
    let error = ProcessController::launch(&mut backend, Path::new("missing-bambu")).unwrap_err();
    assert!(error.to_string().contains("executable was removed"));
}

struct FakeSyncRuntime {
    now: u64,
    snapshots: BTreeMap<PathBuf, VecDeque<Option<Vec<u8>>>>,
    last: BTreeMap<PathBuf, Option<Vec<u8>>>,
    cancelled: bool,
}

impl SyncRuntime for FakeSyncRuntime {
    fn read(&mut self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        let next = self
            .snapshots
            .get_mut(path)
            .and_then(VecDeque::pop_front)
            .unwrap_or_else(|| self.last.get(path).cloned().unwrap_or(None));
        self.last.insert(path.to_path_buf(), next.clone());
        Ok(next)
    }

    fn now_ms(&self) -> u64 {
        self.now
    }

    fn sleep_ms(&mut self, duration: u64) {
        self.now += duration;
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

fn info(setting_id: &str, updated_time: i64) -> Vec<u8> {
    format!(
        "sync_info =\nuser_id =\nsetting_id = {setting_id}\nbase_id =\nupdated_time = {updated_time}\n"
    )
    .into_bytes()
}

fn runtime(paths: &[(&str, Vec<Option<Vec<u8>>>)]) -> FakeSyncRuntime {
    FakeSyncRuntime {
        now: 0,
        snapshots: paths
            .iter()
            .map(|(path, values)| {
                (
                    PathBuf::from(path),
                    values.iter().cloned().collect::<VecDeque<_>>(),
                )
            })
            .collect(),
        last: BTreeMap::new(),
        cancelled: false,
    }
}

fn expectation(id: &str, path: &str, initial: &[u8]) -> SyncExpectation {
    SyncExpectation {
        operation_id: id.to_owned(),
        info_path: PathBuf::from(path),
        initial_sha256: sha256_bytes(initial),
        setting_id_prefix: "PFUS".to_owned(),
    }
}

#[test]
fn sync_monitor_distinguishes_local_load_cloud_id_and_timeout() {
    let initial = info("", 1);
    let loaded = info("", 2);
    let cloud = info("PFUSabc123", 3);
    let expected = [expectation("one", "one.info", &initial)];
    let mut cloud_runtime = runtime(&[(
        "one.info",
        vec![Some(initial.clone()), Some(loaded), Some(cloud)],
    )]);
    let result = SyncMonitor::wait(&expected, &mut cloud_runtime, 1_000, 100).unwrap();
    assert!(!result.timed_out);
    assert_eq!(
        result.observations[0].evidence,
        EvidenceLevel::CloudIdAssigned
    );
    assert_eq!(
        result.observations[0].state,
        OperationState::CloudIdAssigned
    );
    assert_ne!(result.observations[0].evidence, EvidenceLevel::AmsVerified);

    let mut timeout_runtime = runtime(&[(
        "one.info",
        vec![Some(initial.clone()), Some(initial.clone()), Some(initial)],
    )]);
    let timeout = SyncMonitor::wait(&expected, &mut timeout_runtime, 200, 100).unwrap();
    assert!(timeout.timed_out);
    assert_eq!(
        timeout.observations[0].evidence,
        EvidenceLevel::CreatedLocal
    );
    assert_eq!(
        timeout.observations[0].state,
        OperationState::CreatedLocalUnsynchronized
    );
}

#[test]
fn changed_blank_sidecar_is_loaded_by_bambu_without_cloud_claim() {
    let initial = info("", 1);
    let expected = [expectation("one", "one.info", &initial)];
    let mut sync = runtime(&[(
        "one.info",
        vec![Some(initial), Some(info("", 2)), Some(info("", 2))],
    )]);
    let result = SyncMonitor::wait(&expected, &mut sync, 200, 100).unwrap();
    assert_eq!(
        result.observations[0].evidence,
        EvidenceLevel::LoadedByBambu
    );
    assert_eq!(result.observations[0].state, OperationState::LoadedByBambu);
}

#[test]
fn duplicate_cloud_ids_are_reported_and_never_accepted() {
    let initial_a = info("", 1);
    let initial_b = info("", 1);
    let expected = [
        expectation("a", "a.info", &initial_a),
        expectation("b", "b.info", &initial_b),
    ];
    let mut sync = runtime(&[
        ("a.info", vec![Some(info("PFUSduplicate", 2))]),
        ("b.info", vec![Some(info("PFUSduplicate", 2))]),
    ]);
    let result = SyncMonitor::wait(&expected, &mut sync, 100, 100).unwrap();
    assert!(result.timed_out);
    assert!(
        result
            .observations
            .iter()
            .all(|item| item.evidence == EvidenceLevel::LoadedByBambu)
    );
    assert!(result.observations.iter().all(|item| {
        item.diagnostic
            .as_deref()
            .is_some_and(|text| text.contains("duplicate"))
    }));
}

#[test]
fn cancellation_interrupts_sync_wait() {
    let initial = info("", 1);
    let expected = [expectation("one", "one.info", &initial)];
    let mut sync = runtime(&[("one.info", vec![Some(initial)])]);
    sync.cancelled = true;
    let error = SyncMonitor::wait(&expected, &mut sync, 1_000, 100).unwrap_err();
    assert!(error.to_string().contains("cancelled"));
}
