use std::sync::Arc;

use crate::collector::{BytesCodec, Collect};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use super::systeminfo::SystemInfo;

pub struct Collector {
    tx_chan: Sender<Arc<dyn BytesCodec + Send + Sync>>,
    interval: u64, // 每隔period 执行一次
    task: JoinHandle<()>,
}

impl Collect for Collector {
    fn chan(&self) -> Receiver<Arc<dyn BytesCodec + Sync + Send>> {
        self.tx_chan.subscribe()
    }

    fn run(&mut self, _cancellation_token: tokio_util::sync::CancellationToken) {
        let writer = self.tx_chan.clone();
        let mut _scrape_interval =
            tokio::time::interval(tokio::time::Duration::from_secs(self.interval));
        self.task = tokio::spawn(async move {
            for _i in 0..100 {
                let mut system = SystemInfo::new_with_timestampl();
                system.open_fake_proc();
                let _ = writer.send(Arc::new(system));
                tokio::time::sleep(tokio::time::Duration::new(1, 0)).await;
            }
        })
    }
}

impl Collector {
    pub fn new() -> Collector {
        let (tx_sender, _) = broadcast::channel(10);
        Collector {
            interval: 5,
            tx_chan: tx_sender,
            task: tokio::spawn(async {}),
        }
    }
}
