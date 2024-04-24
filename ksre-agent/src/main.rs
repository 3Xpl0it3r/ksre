use libagent::*;

#[tokio::main]
async fn main() {
    let sre_agent = SreAgent::new().await;
    sre_agent.run().await.unwrap();
}
