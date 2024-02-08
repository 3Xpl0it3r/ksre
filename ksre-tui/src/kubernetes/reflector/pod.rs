use color_eyre::eyre::Result;
use futures::{pin_mut, StreamExt};
use k8s_openapi::api::core::v1::{Pod, PodSpec, PodStatus};
use kube::{api::ListParams, runtime::watcher, runtime::watcher::Event, Api, Client};
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::event::KubeEvent;

pub struct PodReflector {
    client: Client,
    task: JoinHandle<()>,
    cancellation_token: CancellationToken,

    tx_event: Sender<KubeEvent<PodSpec, PodStatus>>,
}

impl PodReflector {
    pub fn new(client: Client) -> Result<(Self, Receiver<KubeEvent<PodSpec, PodStatus>>)> {
        let (tx_event, rx_event) = broadcast::channel(1024);
        let mut pod_informer = PodReflector {
            client,
            task: tokio::spawn(async {}),
            cancellation_token: CancellationToken::new(),
            tx_event,
        };
        pod_informer.run().unwrap();
        Ok((pod_informer, rx_event))
    }

    pub fn run(&mut self) -> Result<()> {
        let api = Api::<Pod>::all(self.client.clone());
        let use_watchlist = std::env::var("WATCHLIST")
            .map(|s| s == "1")
            .unwrap_or(false);
        let wc = if use_watchlist {
            watcher::Config::default().streaming_lists()
        } else {
            watcher::Config::default()
        };
        let _cancellation_token = self.cancellation_token.clone();
        let tx_event = self.tx_event.clone();

        self.task = tokio::spawn(async move {
            PodReflector::list_all(&api, ListParams::default(), &tx_event).await;
            let watch_event = watcher(api, wc).fuse();
            pin_mut!(watch_event);
            loop {
                tokio::select! {
                    _ = _cancellation_token.cancelled() => break,
                    event = watch_event.next() => {
                        if let Some(Ok(watch_event)) = event {
                            PodReflector::dispatch_events(&tx_event, watch_event).unwrap();
                        }
                    }
                }
            }
        });
        Ok(())
    }

    fn dispatch_events(
        sender: &Sender<KubeEvent<PodSpec, PodStatus>>,
        watch_event: Event<Pod>,
    ) -> Result<()> {
        match watch_event {
            Event::Applied(applied) => {
                sender.send(KubeEvent::OnAdd(applied.into())).unwrap();
            }
            Event::Deleted(deleted) => {
                sender.send(KubeEvent::OnAdd(deleted.into())).unwrap();
            }
            Event::Restarted(_) => { // the pods thart restart ignored
                 /* sender.send(Event::PodRestart(restart.clone()))?; */
            }
        }
        Ok(())
    }

    async fn list_all(
        api: &Api<Pod>,
        list_opt: ListParams,
        sender: &Sender<KubeEvent<PodSpec, PodStatus>>,
        /* store: RwLockWriteGuard<'_, StoreIndex<PodSpec, PodStatus>>, */
    ) {
        for pod in api.list(&list_opt).await.unwrap() {
            sender.send(KubeEvent::OnAdd(pod.into())).unwrap();
        }
    }
    pub fn shutdown(&mut self) -> Result<()> {
        if !self.cancellation_token.is_cancelled() {
            self.cancellation_token.cancel();
        }
        Ok(())
    }
}

impl Drop for PodReflector {
    fn drop(&mut self) {
        self.shutdown().unwrap()
    }
}
