use color_eyre::Result;
use std::collections::HashMap;
use std::rc::Rc;

use super::api::RtObject;

type Indices<P, U> = HashMap<Rc<str>, HashMap<Rc<str>, Rc<RtObject<P, U>>>>;

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

    pub fn add(&mut self, obj: RtObject<P, U>) -> Result<()> {
        self.update(obj)
    }

    pub fn delete(&mut self, obj: RtObject<P, U>) -> Result<()> {
        let namespace = if let Some(namespace) = obj.0.metadata.namespace.as_deref() {
            Rc::<str>::from(namespace)
        } else {
            Rc::<str>::from("")
        };
        if self.index.get(namespace.as_ref()).is_none() {
            self.index.remove(namespace.as_ref());
            return Ok(());
        }
        let obj = self
            .index
            .get_mut(namespace.as_ref())
            .unwrap()
            .remove(obj.0.metadata.name.as_deref().unwrap());
        if let Some(obj) = obj {
            if let Ok(obj) = Rc::try_unwrap(obj) {
                drop(obj);
            }
        }
        Ok(())
    }
    pub fn update(&mut self, obj: RtObject<P, U>) -> Result<()> {
        let namespace: Rc<str> = if let Some(namespace) = obj.0.metadata.namespace.as_deref() {
            Rc::from(namespace)
        } else {
            Rc::from("")
        };
        let name: Rc<str> = Rc::from(obj.0.metadata.name.as_deref().unwrap());
        if let Some(store) = self.index.get_mut(namespace.as_ref()) {
            store.insert(name, Rc::new(obj));
        } else {
            let store = HashMap::from([(name, Rc::new(obj))]);
            self.index.insert(namespace, store);
        }
        Ok(())
    }

    pub fn all_keys(&self, namespace: &str) -> Vec<Rc<str>> {
        let mut result = Vec::<Rc<str>>::new();
        if namespace.eq("all") {
            for ns in self.index.keys() {
                if let Some(store) = self.index.get(ns) {
                    result.extend(store.keys().cloned());
                }
            }
        } else if let Some(store) = self.index.get(namespace) {
            result.extend(store.keys().cloned());
        }
        result
    }

    pub fn all_values(&self, namespace: &str) -> Vec<Rc<RtObject<P, U>>> {
        let mut result = Vec::<Rc<RtObject<P, U>>>::new();
        if namespace.eq("all") {
            for ns in self.index.keys() {
                if let Some(store) = self.index.get(ns) {
                    result.extend(store.values().cloned());
                }
            }
        } else if let Some(store) = self.index.get(namespace) {
            result.extend(store.values().cloned());
        }
        result
    }

    pub fn get_value(&self, ns: &str, key: &str) -> Option<Rc<RtObject<P, U>>> {
        self.index.get(ns).as_ref()?;

        let store = self.index.get(ns).unwrap();
        if let Some(obj) = store.get(key) {
            Some(obj.clone())
        } else {
            None
        }
    }

    pub fn namespaces(&mut self) -> Vec<Rc<str>> {
        self.index
            .keys()
            .filter(|x| !x.as_ref().eq(""))
            .map(|x| x.clone())
            .collect::<Vec<Rc<str>>>()
    }
}
