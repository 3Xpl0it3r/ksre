use std::sync::Arc;

use crate::collector::Collect;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinHandle;

use super::BytesCodec;

pub struct Collector {
    tx_chan: Sender<Arc<dyn BytesCodec + Send + Sync>>,
    pub result: Vec<String>,
    task: JoinHandle<()>,
}

impl Collect for Collector {
    fn chan(&self) -> tokio::sync::broadcast::Receiver<Arc<dyn BytesCodec + Send + Sync>> {
        self.tx_chan.subscribe()
    }

    fn run(&mut self, _cancellation_token: tokio_util::sync::CancellationToken) {
        println!("Dummy Collector is running");
        let writer = self.tx_chan.clone();
        self.task = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            for i in 0..1000 {
                let key = format!("item-{}", i);
                let _ = writer.send(Arc::new(key));
            }
        });
    }
}

impl Default for Collector {
    fn default() -> Self {
        let (tx_sender, _) = broadcast::channel(1024);
        Collector {
            task: tokio::spawn(async {}),
            tx_chan: tx_sender,
            result: vec![],
        }
    }
}

// Drop[#TODO] (should add some comments)
impl Drop for Collector {
    fn drop(&mut self) {
        if self.task.is_finished() {
            self.task.abort()
        }
    }
}
