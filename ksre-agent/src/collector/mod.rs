use std::sync::Arc;

use tokio_util::sync::CancellationToken;

pub(crate) mod dummy;
pub mod procfs;

use ksre_lib::serializer::bytes::BytesCodec;

// Collector 用来抓取指标,可以丢数据,所以用broadcast 非阻塞的channel
pub trait Collect {
    fn chan(&self) -> tokio::sync::broadcast::Receiver<Arc<dyn BytesCodec + Send + Sync>>;
    fn run(&mut self, cancellation_token: CancellationToken);
}
