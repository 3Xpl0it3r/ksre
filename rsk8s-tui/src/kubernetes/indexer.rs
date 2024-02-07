use color_eyre::Result;
use std::collections::HashMap;
use std::rc::Rc;

use super::api::RtObject;

type Indices<P, U> = HashMap<String, HashMap<String, Rc<RtObject<P, U>>>>;

pub struct StoreIndex<P: Clone, U: Clone> {
    index: Indices<P, U>,
}

impl<P: Clone, U: Clone> Default for StoreIndex<P, U> {
    fn default() -> Self {
        StoreIndex {
            index: HashMap::new(),
        }
    }
}

impl<P: Clone, U: Clone> StoreIndex<P, U> {
    pub fn new() -> Self {
        StoreIndex {
            index: HashMap::new(),
        }
    }

    /* pub fn batch_add(&mut self, obj_list: Vec<RtObject<P, U>>) -> Result<()> {
        for obj in obj_list {
            self.add(obj).unwrap();
        }
        Ok(())
    } */

    pub fn add(&mut self, obj: RtObject<P, U>) -> Result<()> {
        self.update(obj)
    }

    pub fn delete(&mut self, obj: RtObject<P, U>) -> Result<()> {
        let namespace = if let Some(namesapce) = &obj.0.metadata.namespace {
            namesapce.clone()
        } else {
            "".to_string()
        };
        if self.index.get(&namespace).is_none() {
            self.index.remove(&namespace).unwrap();
            return Ok(());
        }

        self.index
            .get_mut(&namespace)
            .unwrap()
            .remove(&obj.0.metadata.name.unwrap());
        Ok(())
    }
    pub fn update(&mut self, obj: RtObject<P, U>) -> Result<()> {
        let namespace = if let Some(namesapce) = &obj.0.metadata.namespace {
            namesapce.clone()
        } else {
            "".to_string()
        };
        let pod_name = format!(
            "{}:{}",
            obj.0.metadata.namespace.as_ref().unwrap(),
            obj.0.metadata.name.as_ref().unwrap()
        );
        if self.index.get(&namespace).is_none() {
            let store = HashMap::from([(pod_name, Rc::new(obj))]);
            self.index.insert(namespace, store);
            Ok(())
        } else {
            let cache = self.index.get_mut(&namespace).unwrap();
            cache.insert(pod_name, Rc::new(obj));
            Ok(())
        }
    }

    pub fn all_keys(&self, namespace: &str) -> Option<Vec<String>> {
        if namespace.eq("all") {
            let mut result = Vec::<String>::new();
            for ns in self.index.keys() {
                result.extend(self.index.get(ns).unwrap().keys().cloned());
            }
            Some(result)
        } else if self.index.get(namespace).is_some() {
            Some(
                self.index
                    .get(namespace)
                    .unwrap()
                    .keys()
                    .cloned()
                    .collect::<Vec<String>>(),
            )
        } else {
            None
        }
    }

    pub fn all_values(&self, ns: &str) -> Option<Vec<Rc<RtObject<P, U>>>> {
        if ns.eq("all") {
            let mut result = Vec::<Rc<RtObject<P, U>>>::new();
            for ns in self.index.keys() {
                result.extend(self.index.get(ns).unwrap().values().cloned());
            }
            Some(result)
        } else {
            Some(
                self.index
                    .get(ns)
                    .unwrap()
                    .values()
                    .cloned()
                    .collect::<Vec<Rc<RtObject<P, U>>>>(),
            )
        }
    }

    pub fn get_value(&self, key: &str) -> Option<Rc<RtObject<P, U>>> {
        let ns_name = key
            .split(':')
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let ns = ns_name.get(0).unwrap();
        let store = self.index.get(ns);
        if store.is_none() {
            return None;
        }
        let store = store.unwrap();
        let obj = store.get(key);
        if obj.is_none() {
            return None;
        }

        let obj = obj.unwrap();

        let ret = obj.clone();
        Some(ret)
    }
}
