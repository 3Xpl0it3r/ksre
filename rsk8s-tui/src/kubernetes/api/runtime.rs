use std::fmt::Debug;

use k8s_openapi::api::{
    apps::v1::{
        DaemonSet, DaemonSetSpec, DaemonSetStatus, Deployment, DeploymentSpec, DeploymentStatus,
        StatefulSet, StatefulSetSpec, StatefulSetStatus,
    },
    core::v1::{Pod, PodSpec, PodStatus, Service, ServiceSpec, ServiceStatus},
};
use kube::core::{Object, TypeMeta};

// RtObject as Kbernetes runtime object
pub struct RtObject<P: Clone, U: Clone>(pub Object<P, U>);

// RtObject<>[#TODO] (should add some comments)
impl<P: Clone, U: Clone> RtObject<P, U> {
    fn resource_name(&mut self) -> String {
        format!(
            "{}:{}",
            self.0.metadata.namespace.as_ref().unwrap(),
            self.0.metadata.name.as_ref().unwrap()
        )
    }
}

// Debug[#TODO] (should add some comments)
impl<P: Clone, U: Clone> Debug for RtObject<P, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entry(self.0.metadata.namespace.as_ref().unwrap())
            .entry(self.0.metadata.name.as_ref().unwrap())
            .finish()
    }
}

// Clone[#TODO] (should add some comments)
impl<P: Clone, U: Clone> Clone for RtObject<P, U> {
    fn clone(&self) -> Self {
        RtObject(Object {
            types: self.0.types.clone(),
            metadata: self.0.metadata.clone(),
            spec: self.0.spec.clone(),
            status: self.0.status.clone(),
        })
    }
}

impl From<Pod> for RtObject<PodSpec, PodStatus> {
    fn from(value: Pod) -> Self {
        const API_VERSION: &'_ str = "v1";
        const KIND: &'_ str = "Pod";
        Self(Object {
            types: Some(TypeMeta {
                api_version: API_VERSION.to_string(),
                kind: KIND.to_string(),
            }),
            metadata: value.metadata,
            spec: value.spec.unwrap(),
            status: value.status,
        })
    }
}

// conversion between RtObject and Deployment
impl From<Deployment> for RtObject<DeploymentSpec, DeploymentStatus> {
    fn from(value: Deployment) -> Self {
        const API_VERSION: &'_ str = "apps/v1";
        const KIND: &'_ str = "Deployment";
        Self(Object {
            types: Some(TypeMeta {
                api_version: API_VERSION.to_string(),
                kind: KIND.to_string(),
            }),
            metadata: value.metadata,
            spec: value.spec.unwrap(),
            status: value.status,
        })
    }
}

impl From<Service> for RtObject<ServiceSpec, ServiceStatus> {
    fn from(value: Service) -> Self {
        const API_VERSION: &'_ str = "v1";
        const KIND: &'_ str = "Service";
        Self(Object {
            types: Some(TypeMeta {
                api_version: API_VERSION.to_string(),
                kind: KIND.to_string(),
            }),
            metadata: value.metadata,
            spec: value.spec.unwrap(),
            status: value.status,
        })
    }
}

impl From<DaemonSet> for RtObject<DaemonSetSpec, DaemonSetStatus> {
    fn from(value: DaemonSet) -> Self {
        const API_VERSION: &'_ str = "apps/v1";
        const KIND: &'_ str = "DaemonSet";
        Self(Object {
            types: Some(TypeMeta {
                api_version: API_VERSION.to_string(),
                kind: KIND.to_string(),
            }),
            metadata: value.metadata,
            spec: value.spec.unwrap(),
            status: value.status,
        })
    }
}
impl From<StatefulSet> for RtObject<StatefulSetSpec, StatefulSetStatus> {
    fn from(value: StatefulSet) -> Self {
        const API_VERSION: &'_ str = "apps/v1";
        const KIND: &'_ str = "StatefulSet";
        Self(Object {
            types: Some(TypeMeta {
                api_version: API_VERSION.to_string(),
                kind: KIND.to_string(),
            }),
            metadata: value.metadata,
            spec: value.spec.unwrap(),
            status: value.status,
        })
    }
}
