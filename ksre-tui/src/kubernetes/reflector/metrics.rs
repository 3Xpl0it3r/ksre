use kube::Client;
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::event::KubeEvent;

pub struct MetricsReflector {
    client: Client,
    task: JoinHandle<()>,
    cancellation_token: CancellationToken,

}
