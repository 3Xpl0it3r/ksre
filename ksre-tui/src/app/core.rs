use std::{ops::Sub, rc::Rc};

use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use k8s_openapi::api::core::v1::{Namespace, PodSpec, PodStatus};
use kube::{api::ListParams, Api, Client as KubeClient, ResourceExt};
use tokio::sync::broadcast;

use crate::{event::KubeEvent, kubernetes::PodMetricsApi, tui::Tui};

use super::{state::Executor, ui::home::ui_main, AppState};

pub struct App {
    tui: Tui,
    kube_client: KubeClient,
    pod_event_rx: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
    app_state: AppState,
    pod_metrics_api: PodMetricsApi,
}

impl App {
    pub fn new(
        tui: Tui,
        kube_event: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
        kube_client: KubeClient,
    ) -> Self {
        let pod_metrics_api = PodMetricsApi::new(kube_client.clone());
        Self {
            tui,
            pod_event_rx: kube_event,
            app_state: AppState::new(),
            kube_client,
            pod_metrics_api,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let namespace: Api<Namespace> = Api::all(self.kube_client.clone());

        for ns in namespace.list(&ListParams::default()).await.unwrap() {
            self.app_state
                .initial_namespaces(Rc::from(ns.name_any().as_str()));
        }
        let mut executor: Option<Executor> = None;
        loop {
            tokio::select! {
                tui_event = self.tui.next()=> {
                    if let Some(event) = tui_event {
                        executor = self.app_state.dispatch_keyevents(event);
                    }
                },
                kube_event = self.pod_event_rx.recv() => {
                    if let Ok(event) = kube_event{
                        executor = self.app_state.handle_podevents(event);
                    }
                },
            }

            if let Some(_executor) = executor.take() {
                _executor.execute();
            }

            if self.app_state.should_quit() {
                break;
            }

            self.draw_ui().await;
        }

        Ok(())
    }

    async fn draw_ui(&mut self) {
        let stdout_buffer = self.app_state.stdout_buffer.clone();
        let reader = stdout_buffer.read().await;
        self.tui
            .draw(|f| ui_main(f, &mut self.app_state, reader))
            .unwrap();
    }
}

// app tempoary task relative
impl App {}

impl Drop for App {
    fn drop(&mut self) {}
}

impl Drop for AppState {
    fn drop(&mut self) {}
}

struct Ticker {
    update_at: DateTime<Utc>,
    curr_time: DateTime<Utc>,
}

// Name[#TODO] (should add some comments)
impl Ticker {
    fn new() -> Self {
        Self {
            update_at: chrono::Utc::now(),
            curr_time: chrono::Utc::now(),
        }
    }

    fn next(&mut self) -> bool {
        let current = self.curr_time;
        self.curr_time = chrono::Utc::now();

        if current.sub(self.update_at).num_seconds() > 1 {
            self.update_at = chrono::Utc::now();
            return true;
        }
        false
    }
}
