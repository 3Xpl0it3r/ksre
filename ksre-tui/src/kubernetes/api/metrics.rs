
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
