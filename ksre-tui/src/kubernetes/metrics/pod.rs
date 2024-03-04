use std::collections::HashMap;

use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube::{api::ListParams, core::ObjectMeta, Api, Client};

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

pub struct MetricClient {
    kube_client: Client,
    namespaced_api: HashMap<String, Api<PodMetrics>>,
    list_api: Api<PodMetrics>,
}

// PodMetricsApi[#TODO] (should add some comments)
impl MetricClient {
    pub fn new(kube_client: Client) -> MetricClient {
        let api = Api::<PodMetrics>::all(kube_client.clone());
        MetricClient {
            kube_client,
            namespaced_api: HashMap::new(),
            list_api: api,
        }
    }

    #[allow(dead_code)]
    pub async fn get(&mut self, namespace: &str, name: &str) -> Option<PodMetrics> {
        if let Some(api) = self.namespaced_api.get(namespace) {
            if let Ok(metric) = api.get(name).await {
                Some(metric)
            } else {
                None
            }
        } else {
            let api = Api::<PodMetrics>::namespaced(self.kube_client.clone(), namespace);
            let result = api.get(name).await;
            self.namespaced_api.insert(namespace.to_string(), api);
            if let Ok(metric) = result {
                Some(metric)
            } else {
                None
            }
        }
    }
    #[allow(dead_code)]
    pub async fn list(&mut self) -> Vec<PodMetrics> {
        self.list_api
            .list(&ListParams::default())
            .await
            .map(|x| x.items)
            .unwrap_or_default()
    }
}
