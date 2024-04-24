use tokio_util::sync::CancellationToken;

pub(crate) mod dummy;
pub(crate) mod procfs;

// Collector 用来抓取指标,可以丢数据,所以用broadcast 非阻塞的channel
pub trait Collector {
    type Item;
    fn chan(&self) -> tokio::sync::broadcast::Receiver<Self::Item>;
    fn run(&mut self, cancellation_token: CancellationToken);
}
