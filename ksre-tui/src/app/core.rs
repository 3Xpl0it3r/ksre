use std::cell::RefCell;
use std::rc::Rc;

use color_eyre::eyre::Result;
use k8s_openapi::api::core::v1::{PodSpec, PodStatus};
use kube::Client as KubeClient;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::event::KubeEvent;
use crate::tui::Tui;

use super::keybind::{KeyContext, DEFAULT_ERROR_HANDLE};
use super::ui::home::ui_main;

use crate::app::job::{pod_exec, PodExecArgs};
use crate::app::AppState;

pub struct App {
    tui: Tui,
    kube_client: KubeClient,
    pod_event_rx: broadcast::Receiver<KubeEvent<PodSpec, PodStatus>>,
    should_quit: bool,
    app_state: AppState,
    task0: JoinHandle<()>,
    task1: JoinHandle<()>,
    cancel_fn: CancellationToken,
    ready: bool,


    cmd_input_writer: Option<mpsc::Sender<String>>,
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
            task0: tokio::spawn(async {}),
            task1: tokio::spawn(async {}),
            cancel_fn: CancellationToken::new(),
            ready: true,
            cmd_input_writer: None,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut keyctx: KeyContext = DEFAULT_ERROR_HANDLE;
        loop {
            tokio::select! {
                tui_event = self.tui.next()=> {
                    if let Some(event) = tui_event {
                        keyctx = self.app_state.handle_terminal_key_event(event);
                    }
                },
                kube_event = self.pod_event_rx.recv() => {
                    if let Ok(event) = kube_event{
                        keyctx = self.app_state.handle_pod_reflect_event(event);
                    }
                },
            }

            // 优先判断是否有cmdcontext , 存在command contxt 意味着当前正在处理command 模式
            if let Some(cmd_context) = self.app_state.consume_command_task() {
                if let Some(handler) = cmd_context.run_fn {
                    handler(self, Some(cmd_context.stop_fn));
                }
            } else if let Some(handler) = keyctx.handler {
                handler(self, None);
                self.ready = true;
            }

            self.draw_ui().await;

            if self.should_quit {
                break;
            }
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

// all handler
impl App {
    pub fn handle_next_route(&mut self, cancel: Option<CancellationToken>) {
        self.app_state.next_route();
    }

    pub fn fake_handlefunction(&mut self, cancel: Option<CancellationToken>) {}

    pub fn handle_tail_pod_log(&mut self, cancel: Option<CancellationToken>) {}

    pub fn handle_quit(&mut self, cancel: Option<CancellationToken>) {
        self.should_quit = true;
    }

    pub fn pod_list_next_item(&mut self, cancel: Option<CancellationToken>) {
        self.app_state.cache_items.next();
    }
    pub fn pod_list_prev_item(&mut self, cancel: Option<CancellationToken>) {
        self.app_state.cache_items.prev();
    }

    pub fn handle_pod_exec(&mut self, cancel: Option<CancellationToken>) {
        if self.ready {
            self.ready = false;
            let cancel = cancel.unwrap();
            let (input_writer, input_reader): (mpsc::Sender<String>, mpsc::Receiver<String>) =
                mpsc::channel(10);
            self.cmd_input_writer = Some(input_writer);
            let pod_args = PodExecArgs {
                kube_client: self.kube_client.clone(),
                pod_name: "default:nginx".to_string(),
                container: None,
            };
            let writer = self.app_state.stdout_buffer_writer();
            self.task1 = tokio::spawn(pod_exec(cancel, writer, input_reader, pod_args));
        } else {
            let writer = self.cmd_input_writer.as_ref().unwrap().clone();
            let input = self.app_state.input_char.clone();
            tokio::spawn(async move {
                writer.send(input).await.expect("send failed");
            });
            self.app_state.input_char.clear();
        }
    }
}

// app tempoary task relative
impl App {}

impl Drop for App {
    fn drop(&mut self) {
        self.should_quit = true;
    }
}

impl Drop for AppState {
    fn drop(&mut self) {}
}
