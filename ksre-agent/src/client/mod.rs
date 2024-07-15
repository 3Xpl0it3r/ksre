pub(crate) mod dummy;

pub(crate) trait Client {
    type Item;
    fn chan(&self) -> tokio::sync::mpsc::Sender<Self::Item>;
    /* fn stop(self) -> Pin<Box<dyn std::future::Future<Output = Result<()>>>>; */
}
