use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::api::AttachParams;
use kube::{Api, Client};
use std::io::Write;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub async fn pod_exec(
    cancellaction_token: CancellationToken,
    kube_client: Client,
    tx_stdout: mpsc::Sender<String>,
    ns_pod: String,
    container: Option<String>,
) {
    let mut nsd_pod = ns_pod.split('/');
    let namespace = nsd_pod.next().unwrap();
    let pod_name = nsd_pod.next().unwrap();

    let pods: Api<Pod> = Api::namespaced(kube_client, namespace);
    let mut attach_params = AttachParams::default()
        .stdin(true)
        .stdout(true)
        .stderr(false)
        .tty(true);
    if let Some(container) = container {
        attach_params = attach_params.container(container);
    }
    let mut attached = pods
        .exec("nginx", vec!["sh"], &attach_params)
        .await
        .unwrap();
    let mut user_input = String::new();
    let mut stdin_writer = attached.stdin().unwrap();
    let mut stdout_stream = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
    loop {
        std::io::stdin().read_line(&mut user_input).unwrap();
        stdin_writer.write_all(user_input.as_bytes()).await.unwrap();
        let next_stdout = stdout_stream.next().await;
        let stdout = String::from_utf8(next_stdout.unwrap().unwrap().to_vec()).unwrap();

        tx_stdout.send(stdout).await.unwrap();

        std::io::stdout().flush().unwrap();
    }
}
