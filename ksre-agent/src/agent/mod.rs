use axum::{routing::get, serve::Serve, Router};
use color_eyre::eyre::Result;
use ksre_lib::serializer::bytes::BytesCodec;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio_util::sync::CancellationToken;

use crate::{
    client::{dummy::DummyClient, Client},
    collector::{procfs::Collector, Collect},
    storage::Store,
};

pub struct Agent {
    collector: Box<dyn Collect>,
    client: Box<dyn Client<Item = String>>,
    server: Serve<Router, Router>,
    store: Store,
}

impl Agent {
    pub async fn new() -> Agent {
        let collector = Collector::new();
        let dummy_client = DummyClient::default();

        let server = Router::new().route("/", get(root));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        let server = axum::serve(listener, server);
        Agent {
            collector: Box::new(collector),
            client: Box::new(dummy_client),
            server,
            store: Store::get_or_create(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let cancellation_token = CancellationToken::default();
        self.collector.run(cancellation_token.clone());

        let mut collector_chan = self.collector.chan();

        let mut count = 0;
        while let Ok(item) = collector_chan.recv().await {
            let buffer = item.byte_encode();
            self.store.append(count, buffer);
            count += 10;
        }

        Ok(())
    }
}
async fn root() -> &'static str {
    "Hello, World!"
}

