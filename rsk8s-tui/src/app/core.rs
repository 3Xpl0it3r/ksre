use color_eyre::eyre::Result;
use k8s_openapi::api::core::v1::{Pod, PodSpec, PodStatus};
use kube::{Api, Client as KubeClient};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::event::KubeEvent;
use crate::tui::Tui;

use super::ui::home::ui_main;
use crate::app::keybind::Handler;

use crate::app::AppState;
pub struct App {
    tui: Tui,
    kube_client: KubeClient,
    pod_event_rx: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
    should_quit: bool,
    /* state: Arc<tokio::sync::RwLock<AppState>>, // may be async?  */
    app_state: AppState,

    ptr_job_0: JoinHandle<()>,
    ptr_job_1: JoinHandle<()>,
    ptr_job_cancel: CancellationToken,
    job_state: bool,
}

impl App {
    pub fn new(
        tui: Tui,
        kube_event: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
        kube_client: KubeClient,
    ) -> Self {
        Self {
            tui,
            pod_event_rx: kube_event,
            kube_client,
            should_quit: false,
            app_state: AppState::default(),
            ptr_job_0: tokio::spawn(async {}),
            ptr_job_1: tokio::spawn(async {}),
            ptr_job_cancel: CancellationToken::new(),
            job_state: false,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        /* self.start_job("kube-system:etcd-minikube".to_string()); */
        loop {
            tokio::select! {
                tui_event = self.tui.next()=> {
                    if let Some(event) = tui_event {
                        self.app_state.handle_key_event(event);
                    }
                },
                kube_event = self.pod_event_rx.recv() => {
                    if kube_event.is_ok() {
                        self.app_state.resync_cache(kube_event.unwrap());
                    }
                },
            }

            self.draw_ui().await;

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }
    fn start_job(&mut self, args: String) {
        self.app_state.empty_stdout_buffer();
        let (writer_tx, mut reader_rx) = mpsc::channel(1024);
        self.ptr_job_cancel = CancellationToken::new();
        self.ptr_job_0 = tokio::spawn(super::job::tail_logs(
            self.ptr_job_cancel.clone(),
            self.kube_client.clone(),
            writer_tx,
            args,
        ));
        let buffer = self.app_state.stdout_buffer.clone();
        let cancellation_token = self.ptr_job_cancel.clone();
        self.ptr_job_1 = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => break,
                    event = reader_rx.recv() => {
                        if let Some(line) = event {
                            let mut writer = buffer.write().await;
                            writer.push(line);
                        }
                    }
                }
            }
        });
    }

    async fn draw_ui(&mut self) {
        let stdout_buffer = self.app_state.stdout_buffer.clone();
        let reader = stdout_buffer.read().await;
        self.tui
            .draw(|f| ui_main(f, &mut self.app_state, reader))
            .unwrap();
    }
}

// Handler[#TODO] (should add some comments)
impl Handler for App {
    fn handle(&mut self) {}
}

fn todo_function() {}

impl Drop for App {
    fn drop(&mut self) {
        self.should_quit = true;
        if !self.ptr_job_cancel.is_cancelled() {
            self.ptr_job_cancel.cancel()
        }
        if !self.ptr_job_1.is_finished() {
            self.ptr_job_1.abort();
        }
        if !self.ptr_job_1.is_finished() {
            self.ptr_job_1.abort()
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {}
}
