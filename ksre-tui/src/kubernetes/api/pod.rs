use std::ops::Deref;
use std::rc::Rc;

use k8s_openapi::{
    api::core::v1::{Container, ContainerState, ContainerStatus, PodSpec, PodStatus, Probe},
    apimachinery::pkg::util::intstr::IntOrString,
};
use kube::core::object::{HasSpec, HasStatus};
use kube::{Resource, ResourceExt};

use super::RtObject;

const DEFAULT_EMPTY_VALUE: &'_ str = "Not Set";

#[derive(Clone, Copy)]
pub struct PodFields<'rc> {
    pub service_account: &'rc str,
    pub node_name: &'rc str,
    /* pub containers: Vec<ContainerFields>,
    pub container_status: Vec<ContainerStatusFields>, */
}

// maybe rc<str> more converience or better  ,but &str more effective for &str cost 16bytes, rc<str> cost 32 bytes
// From[#TODO] (should add some comments)
impl<'rc> From<&'rc Rc<RtObject<PodSpec, PodStatus>>> for PodFields<'rc> {
    fn from(value: &'rc Rc<RtObject<PodSpec, PodStatus>>) -> Self {
        let sa = match value.0.spec.service_account.as_ref() {
            Some(sa) => sa.as_str(),
            None => DEFAULT_EMPTY_VALUE,
        };
        let name = value.0.meta().name.as_ref().unwrap().as_str();
        /* let name = value.0.meta().deref().name.as_deref().unwrap(); */

        PodFields {
            service_account: sa,
            node_name: name,
            /* containers: spec.containers.iter().map(|c| c.into()).collect(),
            container_status: status.iter().map(|x| x.into()).collect(), */
        }
    }
}

#[derive(Clone)]
pub struct ContainerFields<'a> {
    pub name: &'a str,
    // format is {protol}://*:{port}/{path}
    pub liveness_probe: &'a str,
    // format is {protol}://*:{port}/{path}
    pub readness_probe: &'a str,
    pub cpu_limit: &'a str,
    pub cpu_request: &'a str,
    pub mem_limit: &'a str,
    pub mem_requrst: &'a str,
    pub image: &'a str,
}

impl<'a> From<&'a Container> for ContainerFields<'a> {
    // liveness_probe,readness_propbe, cpu_request, cpu_limit, mem_request, mem_limit, start_probe
    fn from(container: &Container) -> ContainerFields {
        todo!()
    }
}

#[derive(Clone)]
pub struct ContainerStatusFields {
    pub name: String,
    pub ready: bool,
    pub restart_count: String,
    pub state: String,
    pub exit_code: String,
    pub message: String,
    pub reason: String,
    pub signal: String,
}

// From<&ContainerStatus>[#TODO] (should add some comments)
impl From<&ContainerStatus> for ContainerStatusFields {
    fn from(value: &ContainerStatus) -> Self {
        let c_state = value.state.as_ref().unwrap();
        let state_fields: ContainerStateFields = c_state.into();
        ContainerStatusFields {
            name: value.name.to_string(),
            ready: value.ready,
            restart_count: value.restart_count.to_string(),
            state: state_fields.state.to_string(),
            exit_code: state_fields.exit_code.unwrap_or(0).to_string(),
            message: state_fields.message.unwrap_or("".to_string()).to_string(),
            reason: state_fields.reason.unwrap_or("".to_string()).to_string(),
            signal: state_fields.signal.unwrap_or(0).to_string(),
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
    use k8s_openapi::api::core::v1::Pod;

    use super::*;
    #[test]
    fn basics() {
        /* let pods = new_fake_pod("pod1", "nodename1", "sa1");
        let fields: PodFields = (&pods).into();
        assert_eq!("nodename1".eq(fields.node_name), true);
        assert_eq!(true, "sa1".eq(fields.service_account)); */
    }
}
