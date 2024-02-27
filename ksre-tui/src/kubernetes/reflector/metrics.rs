use kube::Client;
use tokio::{
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;



pub struct MetricsReflector {
    client: Client,
    task: JoinHandle<()>,
    cancellation_token: CancellationToken,

}
