mod pod_exec;
mod pod_log;

pub use pod_exec::{pod_exec, PodExecArgs};
pub use pod_log::tail_logs;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

enum JobState {
    Init,
    Start,
    Running,
    Stop,
    Terminalted,
    Failed(String),
}

// for JobManager run in single thread , so it don't need lock to procted
struct JobManager {
    task_0: JoinHandle<()>,
    task_1: JoinHandle<()>,
    cancel_fn: CancellationToken,
    state: JobState,
}

impl JobManager {
    pub(super) fn new() -> Self {
        Self {
            task_0: tokio::spawn(async {}),
            task_1: tokio::spawn(async {}),
            cancel_fn: CancellationToken::new(),
            state: JobState::Init,
        }
    }

    pub(super) fn starting(&mut self) {}

    pub(super) fn await_running(&mut self) {}

    pub(super) fn stop(&mut self) {}

    pub(super) fn await_terminated(&mut self) {}
}
