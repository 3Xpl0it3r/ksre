use std::usize;

use super::btree::{BTree, NodeIterator};

// Store[#TODO] (shoule add some comments )
pub struct Store {
    tree: BTree,
}

impl Store {
    pub fn get_or_create() -> Store {
        Store {
            tree: BTree::new("./data"),
        }
    }

    pub fn reader() -> Store {
        Store {
            tree: BTree::reader("./data"),
        }
    }

    pub fn append(&mut self, ts: u64, value: Vec<u8>) {
        self.tree.insert(ts, value)
    }

    pub fn get(&self, time_stamp: u64) -> Option<Vec<u8>> {
        if let Ok(result) = self.tree.fuzz_find(time_stamp) {
            println!("Found : {:?}", result.key);
        }
        None
    }

    pub fn search(&self, start_ts: u64) -> Option<Vec<u8>> {
        if let Ok(result) = self.tree.fuzz_find(start_ts) {
            return Some(result.value);
        }
        None
    }

    pub fn range_query(&self, start_ts: u64, limtis: usize) -> StoreIterator {
        let node_iterator = self.tree.range_iterator(start_ts, limtis);

        StoreIterator {
            node_iter: node_iterator,
        }
    }
}

// Iterator[#TODO] (should add some comments)
impl Iterator for StoreIterator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.node_iter.next()
    }
}

// StoreIterator[#TODO] (shoule add some comments )
pub struct StoreIterator {
    node_iter: NodeIterator,
}
