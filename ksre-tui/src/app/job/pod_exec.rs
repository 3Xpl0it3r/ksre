use std::sync::Arc;

use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::api::AttachParams;
use kube::{Api, Client};
use tokio::{io::AsyncWriteExt, sync::mpsc};
use tokio_util::sync::CancellationToken;
use tui_textarea::TextArea;

pub struct PodExecArgs {
    pub kube_client: Client,
    // namespace:pod
    pub pod_name: String,
    pub container: Option<String>,
}

pub async fn pod_exec(
    cancel: CancellationToken,
    result_writer: Arc<tokio::sync::RwLock<TextArea<'static>>>,
    input_reader: mpsc::Receiver<String>,
    request: PodExecArgs,
) {
    /* let input = request.input.unwrap(); */
    let mut pod_ns_name = request.pod_name.split(':');
    let namespace = pod_ns_name.next().unwrap();
    let pod_name = pod_ns_name.next().unwrap();

    let pods_api: Api<Pod> = Api::namespaced(request.kube_client, namespace);

    let mut attach_opts = default_attached_params();
    if let Some(container) = request.container {
        attach_opts = attach_opts.container(container);
    }
    let mut attached = pods_api
        .exec(pod_name, vec!["sh"], &attach_opts)
        .await
        .unwrap();

    let attached_stdout = attached.stdout().unwrap();
    let attached_stdin = attached.stdin().unwrap();

    let _writer = result_writer.clone();
    let task_0 = tokio::spawn(async move {
        let mut input = input_reader;
        let mut stdin_writer = attached_stdin;
        // if cmd is clear then don't send, only clear buffer
        while let Some(cmd) = input.recv().await {
            if cmd.eq("clear") {
                /* writer.write().await.clear(); */
            } else if stdin_writer
                .write_all(format!("{}\n", cmd).as_bytes())
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let task1 = tokio::spawn(async move {
        let writer = result_writer;
        let mut stdout_stream = tokio_util::io::ReaderStream::new(attached_stdout);
        while let Some(next_output) = stdout_stream.next().await {
            let mut stdout = String::from_utf8(next_output.unwrap().to_vec()).unwrap();
            trim_newline(&mut stdout);
            writer.write().await.insert_str(&stdout);
        }
    });
    cancel.cancelled().await;

    if !task1.is_finished() {
        task1.abort();
    }
    if !task_0.is_finished() {
        task_0.abort();
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\r') {
        s.pop();
    }
}

pub fn default_attached_params() -> AttachParams {
    AttachParams::default()
        .stdin(true)
        .stdout(true)
        .stderr(false)
        .tty(true)
}
