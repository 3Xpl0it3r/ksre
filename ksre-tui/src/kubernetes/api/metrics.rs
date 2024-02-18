struct Metadata {
    name: String,
    namespace: String,
}
pub struct PodMetrics {
    metadata: Metadata,
    containers: Vec<ContainerMetrics>,
}

pub struct ContainerMetrics {
    name: String,
    cpu: String,
    mem: String,
}

pub struct NodeMetrics {
    metadata: Metadata,
}

impl k8s_openapi::Resource for PodMetrics {
    const GROUP: &'static str = "metrics.k8s.io";
    const KIND: &'static str = "PodMetrics";
    const VERSION: &'static str = "v1beta1";
    const API_VERSION: &'static str = "metrics.k8s.io/v1beta1";
    const URL_PATH_SEGMENT: &'static str = "pods";

    type Scope = k8s_openapi::NamespaceResourceScope;
}
