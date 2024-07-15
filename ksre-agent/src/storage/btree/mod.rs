pub mod constant;
pub mod error;
pub mod freelist;
pub mod meta;
pub mod node;
pub mod pager;

mod util;

use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::rc::Rc;
use std::usize;

use constant::{DEFAULT_META_PN, DEFAULT_PAGE_SIZE};
use error::Error;
use freelist::Freelist;
use meta::Meta;
use node::{KeyValue, Node, TypedNode};
use pager::Pager;

use self::node::LeafNode;

// default max size is 40GB for a single tree

pub struct BTree {
    pub pager: Rc<Pager>,
    pub metadata: Meta,
    pub freelist: Freelist,
    read_only: bool,
}

impl BTree {
    pub fn reader(path: &str) -> Self {
        let fp = OpenOptions::new()
            .read(true)
            .open(path)
            .expect("open Btree failed");
        let pager = Pager::new(fp);
        let mut metadata = Meta::default();
        let mut freelist = Freelist::default();
        let mta_page = pager.read_page(DEFAULT_META_PN).unwrap();
        metadata.deserialize(&mta_page.data);

        let fls_page = pager.read_page(metadata.freelist_page).unwrap();
        freelist.deserialize(&fls_page.data);
        BTree {
            pager: Rc::new(pager),
            metadata,
            freelist,
            read_only: true,
        }
    }
    pub fn new(path: &str) -> Self {
        let mut should_initial = false;
        let fp = match OpenOptions::new().write(true).read(true).open(path) {
            Ok(file_ptr) => file_ptr,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    should_initial = true;
                    File::create_new(path).expect("crate new database failed")
                } else {
                    panic!("open database failed");
                }
            }
        };
        let pager = Pager::new(fp);
        let mut metadata = Meta::default();
        let mut freelist = Freelist::default();
        if should_initial {
            let mut meta_page = pager.allocate_page(DEFAULT_META_PN);
            metadata.serialize(&mut meta_page.data);
            pager.write_page(&meta_page);

            let mut fls_page = pager.allocate_page(freelist.get_next_page());
            freelist.serialize(&mut fls_page.data);
            pager.write_page(&fls_page);
        } else {
            let mta_page = pager.read_page(DEFAULT_META_PN).unwrap();
            metadata.deserialize(&mta_page.data);

            let fls_page = pager.read_page(metadata.freelist_page).unwrap();
            freelist.deserialize(&fls_page.data);
        }
        BTree {
            pager: Rc::new(pager),
            metadata,
            freelist,
            read_only: false,
        }
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        let key = key.to_le_bytes();
        if self.metadata.root == 0 {
            let kv = KeyValue::new(&key, value);
            let mut node_page = self.pager.allocate_page(self.freelist.get_next_page());
            let mut new_node = Node::new_leaf(node_page.page_number);
            if let TypedNode::Leaf(ref mut leaf) = new_node.data {
                leaf.keyvalues.push(kv);
                leaf.prev_offset = node_page.page_number;
            }
            new_node.serialize(&mut node_page.data);

            self.write_node(&mut new_node);
            self.metadata.root = new_node.offset;
            self.flush();
            return;
        }

        let mut ancestor_idx = vec![0];
        let (mut node, index, found) = self
            .find_node(self.metadata.root, &key, &mut ancestor_idx)
            .unwrap();

        // node must be leaf node
        if let TypedNode::Leaf(ref mut leaf_node) = node.data {
            if found {
                // directory update
                leaf_node.keyvalues[index].value = value;
            } else {
                let kv = KeyValue::new(&key, value);
                leaf_node.keyvalues.insert(index, kv);
            }
        }

        let mut ancestors = self.get_nodes(&ancestor_idx);

        if node.offset == self.metadata.root {
            ancestors[0] = Rc::new(RefCell::new(node));
        } else {
            ancestors.push(Rc::new(RefCell::new(node)));
        }

        for i in (0..ancestors.len() - 1).rev() {
            let parent = ancestors[i].clone();
            let child = ancestors[i + 1].clone();
            let child_index = ancestor_idx[i + 1];
            if child.borrow().is_overflow() {
                let (mid, mut sibling) = child
                    .borrow_mut()
                    .split(self.freelist.get_next_page())
                    .unwrap();
                parent
                    .borrow_mut()
                    .internal_data()
                    .keys
                    .insert(child_index, mid);
                parent
                    .borrow_mut()
                    .internal_data()
                    .children
                    .insert(child_index + 1, sibling.offset);

                self.write_nodes(&mut [&mut child.borrow_mut(), &mut sibling]);
            } else {
                self.write_node(&mut child.borrow_mut());
            }
        }

        let root_node = ancestors[0].clone();
        if root_node.borrow().is_overflow() {
            let mut new_root = Node::new_internal(self.freelist.get_next_page());
            let (middle_item, mut sibling) = root_node
                .borrow_mut()
                .split(self.freelist.get_next_page())
                .unwrap();

            new_root.internal_data().keys.push(middle_item);
            new_root
                .internal_data()
                .children
                .push(root_node.borrow().offset);
            new_root.internal_data().children.push(sibling.offset);
            self.metadata.root = new_root.offset;

            self.write_nodes(&mut [&mut new_root, &mut root_node.borrow_mut(), &mut sibling]);
            self.flush();
        } else {
            self.write_node(&mut root_node.borrow_mut());
        }
    }

    pub fn delete(&mut self, key: &[u8]) -> Result<String, Error> {
        if self.metadata.root == 0 {
            return Err(Error::EmptyTree);
        }
        let mut ancestor_idx = vec![0];
        let (mut removed_node, removed_index, found) = self
            .find_node(self.metadata.root, key, &mut ancestor_idx)
            .unwrap();
        if !found {
            return Err(Error::KeyNotFound);
        }
        let removed_item = removed_node.leaf_data().keyvalues.remove(removed_index);

        if !found {
            return Err(Error::KeyNotFound);
        }

        let mut ancestors = self.get_nodes(&ancestor_idx);
        if removed_node.offset == self.metadata.root {
            ancestors[0] = Rc::new(RefCell::new(removed_node));
        } else {
            ancestors.push(Rc::new(RefCell::new(removed_node)));
        }

        for i in (0..ancestors.len() - 1).rev() {
            let parent = ancestors[i].clone();
            let child = ancestors[i + 1].clone();
            let child_index = ancestor_idx[i + 1];
            if child.borrow().is_underflow() {
                if child.borrow().is_leaf {
                    self.redistribution_leaf(
                        &mut parent.borrow_mut(),
                        &mut child.borrow_mut(),
                        child_index,
                    );
                } else {
                    self.redistribution_internal(
                        &mut parent.borrow_mut(),
                        &mut child.borrow_mut(),
                        child_index,
                    );
                }
            } else {
                self.write_node(&mut child.borrow_mut());
            }
        }

        let root_node = ancestors.first().unwrap();
        if root_node.borrow().is_leaf {
            // leaf node
            self.write_node(&mut root_node.borrow_mut());
            return Ok(String::from_utf8(removed_item.key).unwrap());
        }

        if root_node.borrow_mut().internal_data().keys.is_empty()
            && root_node.borrow_mut().internal_data().children.len() == 1
        {
            self.metadata.root = root_node
                .borrow_mut()
                .internal_data()
                .children
                .pop()
                .unwrap();
            self.delete_node(root_node.borrow().offset);
        } else {
            self.write_node(&mut root_node.borrow_mut());
        }

        Ok(String::from_utf8(removed_item.key).unwrap())
    }

    // leaf node is underflow, then do re-distribution
    // adopt data from it's neighbor ; then update the parent
    fn redistribution_leaf(
        &mut self,
        parent_node: &mut Node,
        deficient_node: &mut Node,
        deficient_indx: usize,
    ) {
        if deficient_indx > 0 {
            // if deficient node's left sibling exists and has more than minimum number of
            // elements, then rotate right
            let mut l_sibling = self
                .get_node(parent_node.internal_data().children[deficient_indx - 1])
                .unwrap();
            if l_sibling.can_spare_element() {
                // adopt item from left sibling node
                let l_item = l_sibling.leaf_data().keyvalues.pop().unwrap();

                deficient_node.leaf_data().keyvalues.insert(0, l_item);

                // update parent node;
                let new_sep = l_sibling.leaf_data().keyvalues.last().unwrap().key.clone();
                parent_node.internal_data().keys[deficient_indx - 1] = new_sep;

                // persistent nodes
                self.write_node(&mut l_sibling);
                self.write_node(deficient_node);
                return;
            }
        }

        if deficient_indx < parent_node.internal_data().keys.len() - 1 {
            // if deficient node's right sibling exists and has more than minimum number of
            // elements, then rotate left
            let mut r_sibling = self
                .get_node(parent_node.internal_data().children[deficient_indx + 1])
                .unwrap();
            if r_sibling.can_spare_element() {
                // adopt item from right sibling node
                let r_item = r_sibling.leaf_data().keyvalues.remove(0);

                deficient_node.leaf_data().keyvalues.push(r_item);
                // update parent node
                let new_sep = r_sibling.leaf_data().keyvalues.first().unwrap().key.clone();
                parent_node.internal_data().keys[deficient_indx] = new_sep;

                // persistent nodes
                self.write_node(&mut r_sibling);
                self.write_node(deficient_node);
                return;
            }
        }
        // delete the node and merge with neighbor
        if deficient_indx == 0 {
            if let Some(mut r_sibling) = self.get_node(parent_node.internal_data().children[1]) {
                // merge with negihbor
                deficient_node
                    .leaf_data()
                    .keyvalues
                    .append(&mut r_sibling.leaf_data().keyvalues);
                // delete node
                parent_node.internal_data().keys.remove(deficient_indx);
                parent_node
                    .internal_data()
                    .children
                    .remove(deficient_indx + 1);

                deficient_node.leaf_data().next_offset = r_sibling.leaf_data().next_offset;

                self.write_node(deficient_node);
                self.delete_node(r_sibling.offset);
            }
        } else {
            let mut l_sibling = self
                .get_node(parent_node.internal_data().children[deficient_indx - 1])
                .unwrap();
            l_sibling
                .leaf_data()
                .keyvalues
                .append(&mut deficient_node.leaf_data().keyvalues);
            // delete node
            parent_node.internal_data().keys.remove(deficient_indx - 1);
            parent_node.internal_data().children.remove(deficient_indx);

            l_sibling.leaf_data().next_offset = deficient_node.leaf_data().next_offset;

            self.write_node(&mut l_sibling);
            self.delete_node(deficient_node.offset);
        }
    }
    // redistribution internal ,
    // if an internal node ends up with a fewer nodes, underflow
    // adopt from a neighbor ; then update parent
    // if adopt doesn't work, then merge
    fn redistribution_internal(
        &mut self,
        parent_node: &mut Node,
        deficient_node: &mut Node,
        deficient_idx: usize,
    ) {
        // try to rotate from left sibling
        if deficient_idx > 0 {
            // if deficient node's left sibling exists and has more than minimum number of
            // elements, then rotate right
            let mut l_sibling = self
                .get_node(parent_node.internal_data().children[deficient_idx - 1])
                .unwrap();
            if l_sibling.can_spare_element() {
                let old_sep = parent_node.internal_data().keys.remove(deficient_idx - 1);
                let leftest_child = l_sibling.internal_data().children.pop().unwrap();

                deficient_node.internal_data().keys.insert(0, old_sep);
                deficient_node
                    .internal_data()
                    .children
                    .insert(0, leftest_child);

                let new_sep = l_sibling.internal_data().keys.pop().unwrap();

                parent_node
                    .internal_data()
                    .keys
                    .insert(deficient_idx - 1, new_sep);

                self.write_node(&mut l_sibling);
                self.write_node(deficient_node);
                return;
            }
        }

        // try roate from right sibling
        if deficient_idx < parent_node.internal_data().keys.len() - 1 {
            // borrow from right
            // if deficient node's right sibling exists and has more than minimum number of
            // elements, then rotate left
            let mut r_sibling = self
                .get_node(parent_node.internal_data().children[deficient_idx + 1])
                .unwrap();
            if r_sibling.can_spare_element() {
                let old_sep = parent_node.internal_data().keys.remove(deficient_idx);

                let r_first_item = r_sibling.internal_data().keys.remove(0);
                let ship_child = r_sibling.internal_data().children.remove(0);

                parent_node
                    .internal_data()
                    .keys
                    .insert(deficient_idx, r_first_item);

                deficient_node.internal_data().keys.push(old_sep);
                deficient_node.internal_data().children.push(ship_child);

                self.write_node(&mut r_sibling);
                self.write_node(deficient_node);
                return;
            }
        }
        // immediate sibling have only the minimum number of elements, then merge with a sibling
        // sandwiching their separator take off from their parents
        if deficient_idx == 0 {
            if let Some(mut r_sibling) = self.get_node(parent_node.internal_data().children[1]) {
                let old_sep = parent_node.internal_data().keys.remove(deficient_idx);

                deficient_node.internal_data().keys.push(old_sep);
                deficient_node
                    .internal_data()
                    .keys
                    .append(&mut r_sibling.internal_data().keys);
                deficient_node
                    .internal_data()
                    .children
                    .append(&mut r_sibling.internal_data().children);

                parent_node
                    .internal_data()
                    .children
                    .remove(deficient_idx + 1);

                self.write_node(deficient_node);
                self.delete_node(r_sibling.offset);
            }
        } else {
            let mut l_sibling = self
                .get_node(parent_node.internal_data().children[deficient_idx - 1])
                .unwrap();
            let old_sep = parent_node.internal_data().keys.remove(deficient_idx - 1);
            l_sibling.internal_data().keys.push(old_sep);

            l_sibling
                .internal_data()
                .keys
                .append(&mut deficient_node.internal_data().keys);
            l_sibling
                .internal_data()
                .children
                .append(&mut deficient_node.internal_data().children);

            parent_node.internal_data().children.remove(deficient_idx);

            self.write_node(&mut l_sibling);
            self.delete_node(deficient_node.offset);
        }
    }

    fn get_nodes(&self, indexes: &[usize]) -> Vec<Rc<RefCell<Node>>> {
        // return all internalnode
        let mut nodes = vec![];
        let root = self.get_node(self.metadata.root).unwrap();
        nodes.push(Rc::new(RefCell::new(root)));
        if indexes.len() == 1 {
            return nodes;
        }

        for i in 1..indexes.len() - 1 {
            let child_offset =
                nodes[i - 1].clone().borrow_mut().internal_data().children[indexes[i]];
            let child_node = self.get_node(child_offset).unwrap();
            nodes.push(Rc::new(RefCell::new(child_node)));
        }
        nodes
    }

    pub fn range_iterator(&self, key: u64, limits: usize) -> NodeIterator {
        if self.metadata.root == 0 {
            return NodeIterator::none(self.pager.clone(), limits);
        }
        let key = key.to_le_bytes();

        let mut ancestors = vec![];
        if let Ok((node, index, found)) = self.find_node(self.metadata.root, &key, &mut ancestors) {
            if !found && index == 0 {
                return NodeIterator::none(self.pager.clone(), limits);
            }
            if let TypedNode::Leaf(leaf_node) = node.data {
                let index = if found { index } else { index - 1 };
                return NodeIterator {
                    node: Some(leaf_node),
                    index,
                    pager: self.pager.clone(),
                    limits,
                };
            }
        }
        NodeIterator::none(self.pager.clone(), limits)
    }

    pub fn fuzz_find(&self, key: u64) -> Result<KeyValue, Error> {
        if self.metadata.root == 0 {
            return Err(Error::EmptyTree);
        }
        let key = key.to_le_bytes();

        let mut ancestors = vec![];
        if let Ok((node, index, found)) = self.find_node(self.metadata.root, &key, &mut ancestors) {
            if !found && index == 0 {
                return Err(Error::KeyNotFound);
            }
            if let TypedNode::Leaf(ref leaf_node) = node.data {
                let index = if found { index } else { index - 1 };
                return Ok(leaf_node.keyvalues[index].clone());
            }
        }
        Err(Error::KeyNotFound)
    }

    fn find(&self, key: u64) -> Result<Vec<u8>, Error> {
        let key = u64::to_le_bytes(key);
        if self.metadata.root == 0 {
            return Err(Error::EmptyTree);
        }

        let mut ancestors = vec![];
        if let Ok((node, _index, found)) = self.find_node(self.metadata.root, &key, &mut ancestors)
        {
            if found {
                if let TypedNode::Leaf(ref _leaf_node) = node.data {
                    /* return Ok(leaf_node.keyvalues[index].value.clone()); */
                }
            }
        }
        Err(Error::KeyNotFound)
    }

    fn find_node(
        &self,
        node_offset: u64,
        key: &[u8],
        ancestors: &mut Vec<usize>,
    ) -> Result<(Node, usize, bool), Error> {
        let node: Node = if let Some(node) = self.get_node(node_offset) {
            node
        } else {
            return Err(Error::PageLoadErr);
        };

        if node.is_leaf {
            let (found, index) = node.find_key_in_leaf(key);
            Ok((node, index, found))
        } else {
            let (idx, child) = node.find_key_in_internal(key);
            ancestors.push(idx);
            self.find_node(child, key, ancestors)
        }
    }

    fn get_node(&self, page_number: u64) -> Option<Node> {
        let mut node = Node::new_empty(page_number);
        let node_page = self.pager.read_page(page_number).unwrap();
        node.deserialize(&node_page.data);
        node.offset = page_number;
        Some(node)
    }

    pub fn write_nodes(&mut self, nodes: &mut [&mut Node]) {
        for node in nodes {
            self.write_node(node)
        }
    }

    pub fn write_node(&mut self, node: &mut Node) {
        if node.offset == 0 {
            node.offset = self.freelist.get_next_page();
        }
        let mut page = self.pager.allocate_page(node.offset);

        node.serialize(&mut page.data);
        self.pager.write_page(&page);
    }

    pub fn delete_node(&mut self, node: u64) {
        if let Some(mut page) = self.pager.read_page(node) {
            page.data[0..DEFAULT_PAGE_SIZE].clone_from_slice(vec![0; DEFAULT_PAGE_SIZE].as_ref());
            self.pager.write_page(&page);
        }
        self.freelist.release_page(node);
    }

    pub fn flush(&mut self) {
        let mut meta_page = self.pager.allocate_page(DEFAULT_META_PN);
        self.metadata.serialize(&mut meta_page.data);
        self.pager.write_page(&meta_page);

        let mut fls_page = self.pager.allocate_page(self.metadata.freelist_page);
        self.freelist.serialize(&mut fls_page.data);
        self.pager.write_page(&fls_page);
    }
}

// NodeIterator[#TODO] (shoule add some comments )
// BTree[#TODO] (should add some comments)

pub struct NodeIterator {
    node: Option<LeafNode>,
    index: usize,
    pager: Rc<Pager>,
    limits: usize,
}

impl NodeIterator {
    fn none(parer: Rc<Pager>, limits: usize) -> NodeIterator {
        NodeIterator {
            node: None,
            index: 0,
            pager: parer,
            limits,
        }
    }
}

// Iterator[#TODO] (should add some comments)
impl Iterator for NodeIterator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.limits == 0 {
            return None;
        }
        self.node.as_ref()?;
        self.limits -= 1;

        if self.index < self.node.as_ref().unwrap().keyvalues.len() {
            let item = self.node.as_ref().unwrap().keyvalues[self.index].clone();
            self.index += 1;
            return Some(item.value);
        }

        let old_node = self.node.take().unwrap();
        let mut new_node = Node::new_empty(old_node.next_offset);
        let node_page = self.pager.read_page(old_node.next_offset).unwrap();
        new_node.deserialize(&node_page.data);
        match new_node.data {
            TypedNode::Leaf(leaf_node) => {
                self.index = 1;
                let kv = leaf_node.keyvalues[0].clone();
                self.node = Some(leaf_node);
                Some(kv.value)
            }
            _ => None,
        }
    }
}

impl Drop for BTree {
    fn drop(&mut self) {
        if self.read_only {
            return;
        }
        let mut meta_page = self.pager.allocate_page(DEFAULT_META_PN);
        self.metadata.serialize(&mut meta_page.data);
        self.pager.write_page(&meta_page);

        let mut fls_page = self.pager.allocate_page(self.metadata.freelist_page);
        self.freelist.serialize(&mut fls_page.data);
        self.pager.write_page(&fls_page);
    }
}
