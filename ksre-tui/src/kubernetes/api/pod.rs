use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use k8s_openapi::api::core::v1::{ContainerState, PodSpec, PodStatus};
use kube::{
    core::object::{HasSpec, HasStatus},
    Resource,
};

use crate::kubernetes::api::runtime::RtObject;

const NIL_STR: &'_ str = "<none>";

// unsafe
pub struct PodDescribe {
    pub name: *const str,
    pub namespace: *const str,
    pub priority: i32,
    pub service_account: *const str,
    pub labels: String,
    pub node: *const str,
    pub start_time: String,
    pub status: *const str,
    pub ip: *const str,
    pub ips: Vec<*const str>,
    pub qos_class: *const str,
    pub node_selector: String,
    pub containers: Vec<PodDescContainer>,
    pub conditions: HashMap<*const str, *const str>,
    pub ready_number: i32,
}

// unsafe
impl From<&RtObject<PodSpec, PodStatus>> for PodDescribe {
    fn from(object: &RtObject<PodSpec, PodStatus>) -> Self {
        let pod_metadata = object.0.meta();
        let pod_spec = object.0.spec();
        let pod_status = object.0.status();
        let name: *const str = pod_metadata.name.as_deref().unwrap();
        let namespace: *const str = pod_metadata.namespace.as_deref().unwrap();
        let service_account1 = pod_spec.service_account.as_deref().unwrap_or_default();
        let service_account: *const str =
            &(*pod_spec.service_account.as_deref().unwrap_or_default());
        let priority = pod_spec.priority.unwrap_or(0);
        let node: *const str = pod_spec.node_name.as_deref().unwrap_or_default();

        let labels = pod_metadata
            .labels
            .as_ref()
            .map(|x| format!("{:?}", x))
            .unwrap_or_default();

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
                .unwrap_or_default();
            let status = pod_status.phase.as_deref().unwrap_or_default();
            let ip = pod_status.pod_ip.as_deref().unwrap_or_default();
            let ips = pod_status
                .pod_ips
                .as_deref()
                .map(|pod_ips| {
                    pod_ips
                        .iter()
                        .map(|x| {
                            let raw_ptr: *const str =
                                x.ip.as_deref().unwrap_or_default() as *const str;
                            raw_ptr
                        })
                        .collect::<Vec<*const str>>()
                })
                .unwrap_or_default();
            let qos_class = pod_status.qos_class.as_deref().unwrap_or_default();
            let mut running_nr: i32 = 0;
            let containers = pod_status
                .container_statuses
                .as_ref()
                .map(|containers| {
                    containers.iter().map(|container| {
                        let (state, is_running) =
                            container_state_to_hashmap(container.state.as_ref().unwrap());
                        if is_running {
                            running_nr += 1
                        }
                        let (last_state, _) =
                            container_state_to_hashmap(container.last_state.as_ref().unwrap());
                        PodDescContainer {
                            name: container.name.as_str(),
                            container_id: container.container_id.as_deref().unwrap_or_default(),
                            image: container.image.as_str(),
                            image_id: container.image_id.as_str(),
                            state,
                            last_state,
                            started: container.started.unwrap_or_default(),
                            rerestart_count: container.restart_count,
                        }
                    })
                })
                .unwrap()
                .collect::<Vec<PodDescContainer>>();
            let conditions = pod_status
                .conditions
                .as_ref()
                .map(|conditions| {
                    let mut _conditions = HashMap::new();
                    for condition in conditions {
                        _conditions.insert(
                            condition.type_.as_str() as *const str,
                            condition.status.as_str() as *const str,
                        );
                    }
                    _conditions
                })
                .unwrap();

            let name_ptr = std::ptr::addr_of!(name);

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
                ready_number: running_nr,
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
                ready_number: 0,
            }
        }
    }
}

fn container_state_to_hashmap(
    container_state: &ContainerState,
) -> (Vec<(*const str, String)>, bool) {
    if container_state.terminated.is_some() {
        let terminaled_status = container_state.terminated.as_ref().unwrap();
        let mut result = Vec::new();
        result.extend([
            ("State" as *const str, "Terminated".to_string()),
            (
                "ExitCode" as *const str,
                format!("{}", terminaled_status.exit_code),
            ),
            (
                "Finished_At" as *const str,
                terminaled_status
                    .finished_at
                    .as_ref()
                    .map(|x| format!("{:?}", x))
                    .unwrap_or_default(),
            ),
            (
                "Message" as *const str,
                format!(
                    "{:?}",
                    terminaled_status.message.as_deref().unwrap_or_default()
                ),
            ),
            (
                "Reason" as *const str,
                format!(
                    "{:?}",
                    terminaled_status.reason.as_deref().unwrap_or(NIL_STR)
                ),
            ),
            (
                "Signal" as *const str,
                format!("{:?}", terminaled_status.signal.unwrap_or(0)),
            ),
            (
                "Started_At" as *const str,
                terminaled_status
                    .started_at
                    .as_ref()
                    .map(|x| format!("{:?}", x))
                    .unwrap_or_default(),
            ),
        ]);
        return (result, false);
    } else if container_state.running.is_some() {
        let running_state = container_state.running.as_ref().unwrap();
        let mut result = Vec::new();
        result.extend([
            ("State" as *const str, "Running".to_string()),
            (
                "Start_At",
                running_state
                    .started_at
                    .as_ref()
                    .map(|x| format!("{:?}", x))
                    .unwrap_or_default(),
            ),
        ]);
        return (result, true);
    } else if container_state.waiting.is_some() {
        let waiting_state = container_state.waiting.as_ref().unwrap();
        let mut result = Vec::new();
        result.extend([
            ("State" as *const str, "Waiting".to_string()),
            (
                "Reason" as *const str,
                format!("{:?}", waiting_state.reason.as_deref().unwrap_or_default()),
            ),
            (
                "Message" as *const str,
                format!("{:?}", waiting_state.message.as_deref().unwrap_or_default()),
            ),
        ]);
        return (result, false);
    }
    (Vec::new(), false)
}

pub struct PodDescContainer {
    pub name: *const str,
    pub container_id: *const str,
    pub image: *const str,
    pub image_id: *const str,
    pub state: Vec<(*const str, String)>,
    pub last_state: Vec<(*const str, String)>,
    // 此次启动时间
    pub started: bool,
    pub rerestart_count: i32,
}

#[cfg(test)]
mod tests {}
