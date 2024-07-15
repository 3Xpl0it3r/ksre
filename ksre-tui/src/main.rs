use color_eyre::eyre::Result;
use libksre::*;

#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("./", "prefix.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let kube_client = default_kubernetes_client().await?;
    // new instance pod informer, pod_informer will start new coroutine to dispath event from
    // apiserver
    let (mut pod_informer, rx_pod_event) = PodReflector::new(kube_client.clone()).unwrap();
    // new instance tui, tui will start new coroutine to dispatch event from keyboard
    let tui = Tui::new()?;
    // new instance app
    let mut app = App::new(tui, rx_pod_event, kube_client.clone());

    app.run().await.unwrap();
    pod_informer.shutdown()?;

    Ok(())
}
