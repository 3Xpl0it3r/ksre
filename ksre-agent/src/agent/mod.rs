use std::sync::Arc;

use axum::routing::get;
use axum::serve::Serve;
use axum::Router;
use color_eyre::eyre::Result;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use mintkv::db::MintKv;

use crate::client::{dummy::DummyClient, Client};
use crate::collector::procfs::collect::{ProcCollector, ProcT};
use crate::collector::Collector;

pub struct SreAgent {
    collectors: Box<dyn Collector<Item = Arc<ProcT>>>,
    clients: Vec<Box<dyn Client<Item = String>>>,
    server: Serve<Router, Router>,
    storage: MintKv,
}

impl SreAgent {
    pub async fn new() -> SreAgent {
        let proc_collector = ProcCollector::new();

        let dummy_client = DummyClient::default();

        let server = Router::new().route("/", get(root));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        let server = axum::serve(listener, server);
        SreAgent {
            collectors: Box::new(proc_collector),
            clients: vec![Box::new(dummy_client)],
            server,
            storage: MintKv::new("data"),
        }
    }

    pub async fn run(self) -> Result<()> {
        let cancellation_token = CancellationToken::default();

        let mut collector_chan = self.collectors.chan();

        let client_chan_list = self
            .clients
            .iter()
            .map(|client| client.chan())
            .collect::<Vec<mpsc::Sender<String>>>();

        let cancellation_token_cloned = cancellation_token.clone();
        let work_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancellation_token_cloned.cancelled() => break,
                    Ok(item) = collector_chan.recv() => {
                        //save item to databases
                    }
                }
            }
        });

        self.server.await?;

        cancellation_token.cancel(); // stop work_task
        if !work_task.is_finished() {
            work_task.abort();
        }
        cancellation_token.cancelled().await;

        Ok(())
    }
}
async fn root() -> &'static str {
    "Hello, World!"
}
