use color_eyre::eyre::Result;
use rsk8s_lib::*;

#[tokio::main]
async fn main() -> Result<()> {
    let kube_client = default_kubernetes_client().await?;
    // new instance pod informer, pod_informer will start new coroutine to dispath event from
    // apiserver
    let (mut pod_informer, rx_pod_event) = PodReflector::new(kube_client.clone()).unwrap();
    // new instance tui, tui will start new coroutine to dispatch event from keyboard
    let tui = tui::Tui::new()?;
    // new instance app
    let mut app = app::App::new(tui, rx_pod_event, kube_client.clone());

    app.run().await.unwrap();
    pod_informer.shutdown()?;

    Ok(())
}
