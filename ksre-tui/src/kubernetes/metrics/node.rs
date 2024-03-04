use serde::{Deserialize, Serialize};

//for node metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    metadata: kube::api::ObjectMeta,
    usage: Usage,
    timestamp: String,
    window: String,
}
