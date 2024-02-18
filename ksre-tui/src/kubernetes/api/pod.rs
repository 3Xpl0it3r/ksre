use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use k8s_openapi::api::core::v1::{ContainerState, PodSpec, PodStatus};
use kube::{
    core::object::{HasSpec, HasStatus},
    Resource,
};

use crate::kubernetes::api::runtime::RtObject;

const NIL_STR: &'_ str = "<none>";

pub struct PodDescribe<'rc> {
    pub name: &'rc str,
    pub namespace: &'rc str,
    pub priority: i32,
    pub service_account: &'rc str,
    pub labels: String,
    pub node: &'rc str,
    pub start_time: String,
    pub status: &'rc str,
    pub ip: &'rc str,
    pub ips: Vec<&'rc str>,
    pub qos_class: &'rc str,
    pub node_selector: String,
    pub containers: Vec<PodDescContainer<'rc>>,
    pub conditions: HashMap<&'rc str, &'rc str>,
}

// From[#TODO] (should add some comments)
impl<'rc> From<&'rc Rc<RtObject<PodSpec, PodStatus>>> for PodDescribe<'rc> {
    fn from(object: &'rc Rc<RtObject<PodSpec, PodStatus>>) -> Self {
        let pod_metadata = object.0.meta();
        let pod_spec = object.0.spec();
        let pod_status = object.0.status();
        let name = pod_metadata.name.as_ref().unwrap();
        let namespace = pod_metadata.namespace.as_ref().unwrap();
        let priority = pod_spec.priority.unwrap_or(0);
        let service_account = pod_spec.service_account.as_deref().unwrap_or(NIL_STR);
        let node = pod_spec.node_name.as_deref().unwrap_or(NIL_STR);
        let labels = pod_metadata
            .labels
            .as_ref()
            .map(|x| format!("{:?}", x))
            .unwrap_or(NIL_STR.to_string());
        let node_selector = format!(
            "{:?}",
            pod_spec
                .node_selector
                .as_ref()
                .unwrap_or(&BTreeMap::<String, String>::new())
        );
        if pod_status.is_some() {
            let pod_status = pod_status.unwrap();
            let start_time = pod_status
                .start_time
                .as_ref()
                .map(|x| format!("{:?}", x))
                .unwrap_or(NIL_STR.to_string());
            let status = pod_status.phase.as_deref().unwrap_or(NIL_STR);
            let ip = pod_status.pod_ip.as_deref().unwrap_or(NIL_STR);
            let ips = pod_status
                .pod_ips
                .as_deref()
                .map(|pod_ips| {
                    pod_ips
                        .iter()
                        .map(|x| x.ip.as_deref().unwrap_or(NIL_STR))
                        .collect::<Vec<&str>>()
                })
                .unwrap_or_default();
            let qos_class = pod_status.qos_class.as_deref().unwrap_or(NIL_STR);
            let containers = pod_status
                .container_statuses
                .as_ref()
                .map(|containers| {
                    containers.iter().map(|container| PodDescContainer {
                        name: container.name.as_str(),
                        container_id: container.container_id.as_deref().unwrap_or(NIL_STR),
                        image: container.image.as_str(),
                        image_id: container.image_id.as_str(),
                        state: container_state_to_hashmap(container.state.as_ref().unwrap()),
                        last_state: container_state_to_hashmap(
                            container.last_state.as_ref().unwrap(),
                        ),
                        started: container.started.unwrap_or_default(),
                        rerestart_count: container.restart_count,
                    })
                })
                .unwrap()
                .collect::<Vec<PodDescContainer<'rc>>>();
            let conditions = pod_status
                .conditions
                .as_ref()
                .map(|conditions| {
                    let mut _conditions = HashMap::new();
                    for condition in conditions {
                        _conditions.insert(condition.type_.as_str(), condition.status.as_str());
                    }
                    _conditions
                })
                .unwrap();

            Self {
                name,
                namespace,
                priority,
                service_account,
                labels,
                node,
                start_time,
                status,
                ip,
                ips,
                qos_class,
                node_selector,
                containers,
                conditions,
            }
        } else {
            Self {
                name,
                namespace,
                priority,
                service_account,
                labels,
                node,
                start_time: NIL_STR.to_string(),
                status: NIL_STR,
                ip: NIL_STR,
                ips: Vec::new(),
                qos_class: NIL_STR,
                node_selector,
                containers: Vec::new(),
                conditions: HashMap::new(),
            }
        }
    }
}

fn container_state_to_hashmap(container_state: &ContainerState) -> Vec<(&str, String)> {
    if container_state.terminated.is_some() {
        let terminaled_status = container_state.terminated.as_ref().unwrap();
        let mut result = Vec::new();
        result.extend([
            ("State", "Terminated".to_string()),
            ("ExitCode", format!("{}", terminaled_status.exit_code)),
            (
                "Finished_At",
                format!("{:?}", terminaled_status.finished_at),
            ),
            (
                "Message",
                format!(
                    "{:?}",
                    terminaled_status.message.as_deref().unwrap_or(NIL_STR)
                ),
            ),
            (
                "Reason",
                format!(
                    "{:?}",
                    terminaled_status.reason.as_deref().unwrap_or(NIL_STR)
                ),
            ),
            (
                "Signal",
                format!("{:?}", terminaled_status.signal.unwrap_or(0)),
            ),
            ("Started_At", format!("{:?}", terminaled_status.started_at)),
        ]);
        return result;
    } else if container_state.running.is_some() {
        let running_state = container_state.running.as_ref().unwrap();
        let mut result = Vec::new();
        result.extend([
            ("State", "Running".to_string()),
            ("Start_At", format!("{:?}", running_state.started_at)),
        ]);
        return result;
    } else if container_state.waiting.is_some() {
        let waiting_state = container_state.waiting.as_ref().unwrap();
        let mut result = Vec::new();
        result.extend([
            ("State", "Waiting".to_string()),
            (
                "Reason",
                format!("{:?}", waiting_state.reason.as_deref().unwrap_or(NIL_STR)),
            ),
            (
                "Message",
                format!("{:?}", waiting_state.message.as_deref().unwrap_or(NIL_STR)),
            ),
        ]);
        return result;
    }
    Vec::new()
}

pub struct PodDescContainer<'c> {
    pub name: &'c str,
    pub container_id: &'c str,
    pub image: &'c str,
    pub image_id: &'c str,
    pub state: Vec<(&'c str, String)>,
    pub last_state: Vec<(&'c str, String)>,
    // 此次启动时间
    pub started: bool,
    pub rerestart_count: i32,
}

#[cfg(test)]
mod tests {}
