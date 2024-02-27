use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube::core::ObjectMeta;
use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize, Clone, Debug)]
pub struct PodMetricsContainer {
    pub name: String,
    pub usage: PodMetricsContainerUsage,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct PodMetricsContainerUsage {
    pub cpu: Quantity,
    pub memory: Quantity,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct PodMetrics {
    pub metadata: ObjectMeta,
    pub timestamp: String,
    pub window: String,
    pub containers: Vec<PodMetricsContainer>,
}

impl k8s_openapi::Resource for PodMetrics {
    const GROUP: &'static str = "metrics.k8s.io";
    const KIND: &'static str = "PodMetrics";
    const VERSION: &'static str = "v1beta1";
    const API_VERSION: &'static str = "metrics.k8s.io/v1beta1";
    const URL_PATH_SEGMENT: &'static str = "pods";

    type Scope = k8s_openapi::NamespaceResourceScope;
}

impl k8s_openapi::Metadata for PodMetrics {
    type Ty = ObjectMeta;

    fn metadata(&self) -> &Self::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut Self::Ty {
        &mut self.metadata
    }
}

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
