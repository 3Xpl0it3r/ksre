use crate::collector::Collector;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinHandle;

pub struct DummyCollector {
    tx_chan: Sender<String>,
    pub result: Vec<String>,
    task: JoinHandle<()>,
}

impl Collector for DummyCollector {
    type Item = String;

    fn chan(&self) -> tokio::sync::broadcast::Receiver<Self::Item> {
        self.tx_chan.subscribe()
    }

    fn run(&mut self, cancellation_token: tokio_util::sync::CancellationToken) {}
}

impl Default for DummyCollector {
    fn default() -> Self {
        let (tx_sender, _) = broadcast::channel(1024);
        let writer = tx_sender.clone();
        let task = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            for i in 0..1000 {
                writer.send(format!("item-{}", i)).unwrap();
            }
        });
        DummyCollector {
            task,
            tx_chan: tx_sender,
            result: vec![],
        }
    }
}

// Drop[#TODO] (should add some comments)
impl Drop for DummyCollector {
    fn drop(&mut self) {
        if self.task.is_finished() {
            self.task.abort()
        }
    }
}
