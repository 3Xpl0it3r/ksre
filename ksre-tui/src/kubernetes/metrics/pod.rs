use std::collections::HashMap;

use kube::api::ListParams;
use kube::{Api, Client};

use crate::kubernetes::api::metrics::PodMetrics;

pub struct PodMetricsApi {
    kube_client: Client,
    namespaced_api: HashMap<String, Api<PodMetrics>>,
    list_api: Api<PodMetrics>,
}

// PodMetricsApi[#TODO] (should add some comments)
impl PodMetricsApi {
    pub fn new(kube_client: Client) -> PodMetricsApi {
        let api = Api::<PodMetrics>::all(kube_client.clone());
        PodMetricsApi {
            kube_client,
            namespaced_api: HashMap::new(),
            list_api: api,
        }
    }

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
    pub async fn list(&mut self) -> Vec<PodMetrics> {
        self.list_api
            .list(&ListParams::default())
            .await
            .map(|x| x.items)
            .unwrap_or_default()
    }
}
