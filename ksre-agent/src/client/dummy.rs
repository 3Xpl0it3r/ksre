use crate::client::Client;
use tokio::sync::mpsc::{self, Sender};
use tokio::task::JoinHandle;

pub struct DummyClient {
    // receive message from rx_reader, then send to clients
    tx_sender: Sender<String>,
    pub fake_server: Vec<u8>,
    task_handler: JoinHandle<()>,
}

impl Client for DummyClient {
    type Item = String;
    fn chan(&self) -> tokio::sync::mpsc::Sender<Self::Item> {
        self.tx_sender.clone()
    }
}

impl Default for DummyClient {
    fn default() -> Self {
        let (tx_writer, mut rx_reader) = mpsc::channel(1024);
        let task_handler = tokio::spawn(async move {
            while let Some(buffer) = rx_reader.recv().await {
                println!("Client receive : {:?}", buffer);
            }
        });
        DummyClient {
            task_handler,
            tx_sender: tx_writer,
            fake_server: vec![],
        }
    }
}

impl Drop for DummyClient {
    fn drop(&mut self) {
        if !self.task_handler.is_finished() {
            self.task_handler.abort();
        }
    }
}
