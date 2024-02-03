use futures::{AsyncBufReadExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::LogParams;
use kube::{Api, Client as KubeClient};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub async fn tail_logs(
    cancellation_token: CancellationToken,
    kube_client: KubeClient,
    tx_stdout: mpsc::Sender<String>,
    ns_pod: String,
) {
    let mut nsd_pod = ns_pod.split('/');
    let namespace = nsd_pod.next().unwrap();
    let pod_name = nsd_pod.next().unwrap();

    let pods: Api<Pod> = Api::namespaced(kube_client, namespace);
    let log_opts = LogParams::default();
    let mut log_stream = pods.log_stream(pod_name, &log_opts).await.unwrap().lines();

    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => break,
            maybe_log = log_stream.try_next() => {
                if let Ok(Some(line)) = maybe_log {
                    tx_stdout.send(line).await.unwrap();
                }
            }
        }
    }
}
