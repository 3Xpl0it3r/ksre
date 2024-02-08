use color_eyre::eyre::Result;
use kube::Client;

pub async fn default_kubernetes_client() -> Result<Client> {
    Ok(Client::try_default().await.unwrap())
}
