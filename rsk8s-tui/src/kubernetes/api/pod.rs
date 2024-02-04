use std::rc::Rc;

use k8s_openapi::api::core::v1::{
    Container, ContainerState, ContainerStatus, Pod, PodSpec, PodStatus, Probe,
};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;

use super::RtObject;

#[derive(Clone)]
pub struct PodFields {
    pub service_account: String,
    pub node_name: String,
    pub containers: Vec<ContainerFields>,
    pub container_status: Vec<ContainerStatusFields>,
}

// From[#TODO] (should add some comments)
impl From<Rc<RtObject<PodSpec, PodStatus>>> for PodFields {
    fn from(value: Rc<RtObject<PodSpec, PodStatus>>) -> Self {
        let spec = &value.0.spec;
        let status = value
            .0
            .status
            .as_ref()
            .unwrap()
            .container_statuses
            .as_ref()
            .unwrap();

        let sa = match spec.service_account.as_ref() {
            Some(sa) => sa.to_owned(),
            None => "not found".to_owned(),
        };
        let node_name = match spec.node_name.as_ref() {
            Some(node) => node.to_owned(),
            None => "not found".to_owned(),
        };

        /* let a = status.container_statuses.as_ref().unwrap().iter().map(|x|x.into()).collect(); */
        PodFields {
            service_account: sa,
            node_name,
            containers: spec.containers.iter().map(|c| c.into()).collect(),
            container_status: status.iter().map(|x| x.into()).collect(),
        }
    }
}

#[derive(Clone)]
pub struct ContainerFields {
    pub name: String,
    // format is {protol}://*:{port}/{path}
    pub liveness_probe: Option<String>,
    // format is {protol}://*:{port}/{path}
    pub readness_probe: Option<String>,
    pub cpu_limit: Option<String>,
    pub cpu_request: Option<String>,
    pub mem_limit: Option<String>,
    pub mem_requrst: Option<String>,
    pub image: String,
}

impl From<&Container> for ContainerFields {
    fn from(container: &Container) -> ContainerFields {
        let probe_gather = |probe: &Probe| -> Option<String> {
            if probe.grpc.is_some() {
                let grpc = probe.grpc.as_ref().unwrap();
                if let Some(service) = grpc.service.as_ref() {
                    return Some(format!("grpc://*:{}/{}", grpc.port, service));
                }
                return Some(format!("grpc://*:{}", grpc.port));
            }
            if probe.http_get.is_some() {
                let http_action = probe.http_get.as_ref().unwrap();
                return Some(format!(
                    "{}://{}:{}/{}",
                    http_action.scheme.as_ref().unwrap_or(&"http".to_string()),
                    http_action.host.as_ref().unwrap_or(&"*".to_string()),
                    match &http_action.port {
                        IntOrString::Int(int) => int.to_string(),
                        IntOrString::String(string) => string.clone(),
                    },
                    http_action.path.as_ref().unwrap_or(&"".to_string())
                ));
            }

            None
        };
        ContainerFields {
            name: container.name.to_owned(),
            image: container.image.as_ref().unwrap().to_owned(),
            liveness_probe: if container.liveness_probe.is_some() {
                probe_gather(container.liveness_probe.as_ref().unwrap())
            } else {
                None
            },
            readness_probe: if container.readiness_probe.is_some() {
                probe_gather(container.readiness_probe.as_ref().unwrap())
            } else {
                None
            },
            cpu_limit: None,
            cpu_request: None,
            mem_limit: None,
            mem_requrst: None,
        }
    }
}

#[derive(Clone)]
pub struct ContainerStatusFields {
    pub name: String,
    pub ready: bool,
    pub restart_count: i32,
    pub state: String,
    pub exit_code: Option<i32>,
    pub message: Option<String>,
    pub reason: Option<String>,
    pub signal: Option<i32>,
}

// From<&ContainerStatus>[#TODO] (should add some comments)
impl From<&ContainerStatus> for ContainerStatusFields {
    fn from(value: &ContainerStatus) -> Self {
        let c_state = value.state.as_ref().unwrap();
        let state_fields: ContainerStateFields = c_state.into();
        ContainerStatusFields {
            name: value.name.to_string(),
            ready: value.ready,
            restart_count: value.restart_count,
            state: state_fields.state,
            exit_code: state_fields.exit_code,
            message: state_fields.message,
            reason: state_fields.reason,
            signal: state_fields.signal,
        }
    }
}

struct ContainerStateFields {
    state: String,
    exit_code: Option<i32>,
    message: Option<String>,
    reason: Option<String>,
    signal: Option<i32>,
}

// From[#TODO] (should add some comments)
impl From<&ContainerState> for ContainerStateFields {
    fn from(container_state: &ContainerState) -> Self {
        if container_state.running.is_some() {
            ContainerStateFields {
                state: "running".to_string(),
                exit_code: None,
                message: None,
                reason: None,
                signal: None,
            }
        } else if container_state.terminated.is_some() {
            let ref_terminated = container_state.terminated.as_ref().unwrap();
            ContainerStateFields {
                state: "terminated".to_string(),
                exit_code: Some(ref_terminated.exit_code),
                message: ref_terminated.message.clone(),
                reason: ref_terminated.reason.clone(),
                signal: ref_terminated.signal,
            }
        } else {
            let ref_waitting = container_state.waiting.as_ref().unwrap();
            ContainerStateFields {
                state: "waiting".to_string(),
                exit_code: None,
                message: ref_waitting.message.clone(),
                reason: ref_waitting.reason.clone(),
                signal: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    fn new_fake_pod(pod_name: &str, node_name: &str, service_account: &str) -> Pod {
        let mut pod = Pod::default();
        if pod.spec.is_none() {
            pod.spec = Some(PodSpec::default());
        }
        pod.metadata.name = Some(format!("pod_name_{}", pod_name.to_owned()));
        pod.spec.as_mut().unwrap().node_name = Some(node_name.to_owned());
        pod.spec.as_mut().unwrap().service_account = Some(service_account.to_owned());
        pod
    }
    use super::*;
    #[test]
    fn basics() {
        /* let pods = new_fake_pod("pod1", "nodename1", "sa1");
        let fields: PodFields = (&pods).into();
        assert_eq!("nodename1".eq(fields.node_name), true);
        assert_eq!(true, "sa1".eq(fields.service_account)); */
    }
}
