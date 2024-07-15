use core::panic;
use std::{u64, usize};

use super::constant::{
    DEFAULT_MAX_THRESHOLD, DEFAULT_MIN_THRESHOLD, HEAD_INTERNAL_NODE_SIZE, HEAD_LEAF_NODE_SIZE,
};
use super::error::Error;
use super::util;

type Offset = u64;
type Key = Vec<u8>;
type Value = Vec<u8>;

#[derive(Default, Clone, Debug)]
pub struct KeyValue {
    pub key: Key,
    pub value: Value,
}

// KeyValue[#TODO] (should add some comments)
impl KeyValue {
    pub fn new(key: &[u8], value: Vec<u8>) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }
}

pub enum TypedNode {
    Internal(InternalNode),
    Leaf(LeafNode),
    Empty,
}

// LeafNode[#TODO] (shoule add some comments )
#[derive(Default)]
pub struct LeafNode {
    pub keyvalues: Vec<KeyValue>,
    pub prev_offset: Offset,
    pub next_offset: Offset,
}

#[derive(Default)]
pub struct InternalNode {
    pub keys: Vec<Key>,
    pub children: Vec<Offset>,
}

const NODEASLEAF: u8 = 0;
const NODEASINTERNAL: u8 = 1;

pub struct Node {
    pub offset: Offset, // cost 8B
    pub data: TypedNode,
    pub is_leaf: bool,
}

// Node[#TODO] (should add some comments)
impl Node {
    pub fn new_empty(offset: Offset) -> Self {
        Self {
            offset,
            data: TypedNode::Empty,
            is_leaf: true,
        }
    }
    pub fn new_leaf(offset: Offset) -> Self {
        Self {
            offset,
            data: TypedNode::Leaf(LeafNode::default()),
            is_leaf: true,
        }
    }
    pub fn new_internal(offset: Offset) -> Self {
        Self {
            offset,
            data: TypedNode::Internal(InternalNode::default()),
            is_leaf: false,
        }
    }

    pub fn leaf_data(&mut self) -> &mut LeafNode {
        if let TypedNode::Leaf(ref mut leaf_node) = self.data {
            leaf_node
        } else {
            panic!("Expected leaf node, But got internal node");
        }
    }

    pub fn internal_data(&mut self) -> &mut InternalNode {
        if let TypedNode::Internal(ref mut internal_node) = self.data {
            internal_node
        } else {
            panic!("Expected internal node, But got leaf node");
        }
    }
}

///  node format , value is offset if the node is internal node, else value is String
/// |-----------------------------------------------------------------------------------|
/// | node type | key number  |   .............. Data ....                              |
/// |  1B       |  2B         |   ........................                              |
/// |-----------------------------------------------------------------------------------|
/// |                          DATA                                                     |
/// | c0 | len(k0) |  k0 | len(v0)  | v0 |  c1 |  len(k1) | k2 | len(v2) | v2 | ........|
/// | 8B |  1B     | xx  |  1B      | xx |  8B |  1B      | x  |  1B     | xxx| ........|
/// |-----------------------------------------------------------------------------------|
///
impl Node {
    pub fn serialize(&self, buf: &mut [u8]) {
        match self.data {
            TypedNode::Internal(ref internal_node) => self.serialize_internal(internal_node, buf),
            TypedNode::Leaf(ref leaf_node) => self.serialize_leaf(leaf_node, buf),
            _ => {}
        };
    }

    fn serialize_internal(&self, internal_node: &InternalNode, buf: &mut [u8]) {
        let mut offset = 0;
        // write node type, which cost 1B
        buf[offset] = NODEASINTERNAL;
        offset += 1;

        // write numbers of keys, which cost 8B
        let key_num = internal_node.keys.len() as u64;
        buf[offset..offset + 8].clone_from_slice(u64::to_le_bytes(key_num).as_ref());
        offset += 8;

        // write keys into page
        for key in internal_node.keys.iter() {
            // write the length of keys into page , which cost 2B
            let key_size = key.len();
            buf[offset..offset + 2].clone_from_slice(u16::to_le_bytes(key_size as u16).as_ref());
            offset += 2;
            // write key into page, which cost key_size
            buf[offset..offset + key_size].clone_from_slice(key);
            offset += key_size;
        }

        // write number of child into page, which cost 8B
        let child_num = internal_node.children.len() as u64;
        buf[offset..offset + 8].clone_from_slice(u64::to_le_bytes(child_num).as_ref());
        offset += 8;

        // write children into page, which child cost 8B
        for &child in internal_node.children.iter() {
            buf[offset..offset + 8].clone_from_slice(u64::to_le_bytes(child).as_ref());
            offset += 8;
        }
    }

    fn serialize_leaf(&self, leaf_node: &LeafNode, buf: &mut [u8]) {
        let mut offset = 0;
        // write node type, which cost 1B
        buf[offset] = NODEASLEAF;
        offset += 1;

        // write prev_offset, which cost 8B
        buf[offset..offset + 8].clone_from_slice(u64::to_le_bytes(leaf_node.prev_offset).as_ref());
        offset += 8;

        // write next_offset, which cost 8B
        buf[offset..offset + 8].clone_from_slice(u64::to_le_bytes(leaf_node.next_offset).as_ref());
        offset += 8;

        // write numbers of keys, which cost 8B
        let key_num = leaf_node.keyvalues.len() as u64;
        buf[offset..offset + 8].clone_from_slice(u64::to_le_bytes(key_num).as_ref());
        offset += 8;

        // write key value into page
        for kv in leaf_node.keyvalues.iter() {
            // write key size into page,which cost 2B
            let key_size = kv.key.len();
            buf[offset..offset + 2].clone_from_slice(u16::to_le_bytes(key_size as u16).as_ref());
            offset += 2;

            // write key into page
            buf[offset..offset + key_size].clone_from_slice(kv.key.as_ref());
            offset += key_size;

            // write value size into page, which cost 2B
            let value_size = kv.value.len();
            buf[offset..offset + 2].clone_from_slice(u16::to_le_bytes(value_size as u16).as_ref());
            offset += 2;

            // write value bytes into page, which cost value_size * B
            buf[offset..offset + value_size].clone_from_slice(kv.value.as_ref());
            offset += value_size;
        }
    }

    pub fn deserialize(&mut self, buf: &[u8]) {
        // get node type at first Byte
        let ntype = buf[0];
        if ntype == NODEASLEAF {
            self.deserialize_leaf(buf);
        } else {
            self.deserialize_internal(buf);
        }
    }
    fn deserialize_internal(&mut self, buf: &[u8]) {
        let mut offset = 0;
        self.is_leaf = buf[offset] == NODEASLEAF;
        offset += 1;

        // get number of keys, which should read 8B from buffer
        let keys_num = u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
        offset += 8;
        // get keys from buffer,
        let mut keys = vec![];

        for _ in 0..keys_num {
            // get size of key, which should be read 2B
            let key_size = u16::from_le_bytes(buf[offset..offset + 2].try_into().unwrap()) as usize;
            offset += 2;

            // get key String from buffer, which should be read key_size * B
            let mut key_bytes = vec![0; key_size];
            key_bytes.clone_from_slice(buf[offset..offset + key_size].into());
            offset += key_size;

            keys.push(key_bytes);
        }
        // get children
        let mut children = vec![];
        let children_num = u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
        offset += 8;
        for _ in 0..children_num {
            let child = u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
            offset += 8;
            children.push(child);
        }

        self.data = TypedNode::Internal(InternalNode { keys, children });
    }

    fn deserialize_leaf(&mut self, buf: &[u8]) {
        let mut offset = 0;

        // read nodetype
        self.is_leaf = buf[offset] == NODEASLEAF;
        offset += 1;

        // get preoffset
        let prev_offset = u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
        offset += 8;

        //
        // get nextoffset
        let next_offset = u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let kv_num = u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let mut key_values = Vec::with_capacity(kv_num as usize);
        for _ in 0..kv_num {
            // get key size, which we should read 2B from buffer
            let key_size = u16::from_le_bytes(buf[offset..offset + 2].try_into().unwrap()) as usize;
            offset += 2;

            // get key bytes, which we should read key_size * B from buffer
            let mut key_bytes = vec![0; key_size];
            key_bytes.clone_from_slice(buf[offset..offset + key_size].try_into().unwrap());
            offset += key_size;

            // get value size, which we should read 2B from buffer
            let val_size = u16::from_le_bytes(buf[offset..offset + 2].try_into().unwrap()) as usize;
            offset += 2;

            // get value bytes, which we should read value_size * B from buffer
            let mut value_bytes = vec![0; val_size];
            value_bytes.clone_from_slice(buf[offset..offset + val_size].try_into().unwrap());
            offset += val_size;

            key_values.push(KeyValue {
                key: key_bytes,
                value: value_bytes,
            });
        }
        self.data = TypedNode::Leaf(LeafNode {
            keyvalues: key_values,
            prev_offset,
            next_offset,
        });
    }

    pub fn split(&mut self, new_offset: Offset) -> Result<(Key, Node), Error> {
        let split_index = self.get_split_index();
        if split_index == -1 {
            return Err(Error::Generic);
        }
        let splited_index = split_index as usize;
        match self.data {
            TypedNode::Internal(ref mut internal_node) => {
                let middle_item = internal_node.keys[splited_index].clone();
                let mut new_node = Node::new_internal(new_offset);
                new_node
                    .internal_data()
                    .keys
                    .extend_from_slice(internal_node.keys[splited_index + 1..].as_ref());
                internal_node.keys.drain(splited_index..);
                new_node
                    .internal_data()
                    .children
                    .extend_from_slice(internal_node.children[splited_index + 1..].as_ref());
                internal_node.children.drain(splited_index + 1..);

                Ok((middle_item, new_node))
            }
            TypedNode::Leaf(ref mut leaf_node) => {
                if leaf_node.keyvalues.len() < 2 {
                    return Err(Error::Generic);
                }
                let splited_index = 1;
                let middle_item = leaf_node.keyvalues[splited_index].clone();
                let mut new_node = Node::new_leaf(new_offset);
                new_node
                    .leaf_data()
                    .keyvalues
                    .extend_from_slice(&leaf_node.keyvalues[splited_index..]);
                leaf_node.keyvalues.drain(splited_index..);
                leaf_node.next_offset = new_node.offset;
                new_node.leaf_data().prev_offset = self.offset;
                Ok((middle_item.key, new_node))
            }
            TypedNode::Empty => Err(Error::Generic),
        }
    }

    pub fn can_spare_element(&self) -> bool {
        match self.data {
            TypedNode::Internal(ref internal_node) => internal_node.keys.len() > 1,
            TypedNode::Leaf(ref leaf_node) => leaf_node.keyvalues.len() > 1,
            TypedNode::Empty => todo!(),
        }
    }

    pub fn find_key_in_leaf(&self, key: &[u8]) -> (bool, usize) {
        if let TypedNode::Leaf(ref leaf_node) = self.data {
            for (idx, elem) in leaf_node.keyvalues.iter().enumerate() {
                match util::bytes_compare(&elem.key, key) {
                    std::cmp::Ordering::Equal => {
                        return (true, idx);
                    }
                    std::cmp::Ordering::Greater => {
                        return (false, idx);
                    }
                    std::cmp::Ordering::Less => {}
                }
            }
            (false, leaf_node.keyvalues.len())
        } else {
            panic!("this is not leaf node");
        }
    }

    pub fn find_key_in_internal(&self, key: &[u8]) -> (usize, Offset) {
        if let TypedNode::Internal(ref internal_node) = self.data {
            for (idx, elem) in internal_node.keys.iter().enumerate() {
                /* match bytes::compare(elem, key) { */
                match util::bytes_compare(elem, key) {
                    std::cmp::Ordering::Equal => {
                        return (idx + 1, internal_node.children[idx + 1]);
                    }
                    std::cmp::Ordering::Greater => {
                        return (idx, internal_node.children[idx]);
                    }
                    std::cmp::Ordering::Less => {}
                }
            }
            (
                internal_node.keys.len(),
                *internal_node.children.last().unwrap(),
            )
        } else {
            panic!("this is not internal node");
        }
    }

    fn get_split_index(&self) -> i32 {
        match self.data {
            TypedNode::Internal(ref internal_node) => {
                let mut threshold_value = HEAD_INTERNAL_NODE_SIZE;
                for idx in 0..internal_node.keys.len() {
                    threshold_value += internal_node.keys[idx].len() + 2 + 8;
                    if threshold_value > DEFAULT_MIN_THRESHOLD as usize {
                        return idx as i32;
                    }
                }
                -1
            }
            TypedNode::Leaf(ref leaf_node) => {
                // 8B for the last child
                let mut threshold_value = HEAD_LEAF_NODE_SIZE + 8;
                for (idx, kv) in leaf_node.keyvalues.iter().enumerate() {
                    threshold_value += kv.key.len() + kv.value.len() + 4;
                    if threshold_value > DEFAULT_MIN_THRESHOLD as usize {
                        return idx as i32;
                    }
                }
                -1
            }
            TypedNode::Empty => -1,
        }
    }

    pub fn is_underflow(&self) -> bool {
        match self.data {
            TypedNode::Internal(ref internal_node) => {
                let mut threshold_value = HEAD_INTERNAL_NODE_SIZE;
                for idx in 0..internal_node.keys.len() {
                    threshold_value += internal_node.keys[idx].len() + 2 + 8;
                }
                threshold_value < DEFAULT_MIN_THRESHOLD as usize
            }
            TypedNode::Leaf(ref leaf_node) => {
                let mut threshold_value = HEAD_LEAF_NODE_SIZE + 8;
                for kv in leaf_node.keyvalues.iter() {
                    threshold_value += kv.key.len() + kv.value.len() + 4;
                }
                threshold_value < DEFAULT_MIN_THRESHOLD as usize
            }
            TypedNode::Empty => todo!(),
        }
        /* self.leaf_items.len() < max_kvs().div(2) */
    }

    pub fn is_overflow(&self) -> bool {
        match self.data {
            TypedNode::Internal(ref internal_node) => {
                let mut threshold_value = HEAD_INTERNAL_NODE_SIZE;
                for idx in 0..internal_node.keys.len() {
                    /* if idx > DEFAULT_MAX_LEAF_ITEMS_NUM as usize {
                        return true;
                    } */
                    threshold_value += internal_node.keys[idx].len() + 2 + 8;
                }
                (threshold_value as f64) > DEFAULT_MAX_THRESHOLD
            }
            TypedNode::Leaf(ref leaf_node) => {
                let mut threshold_value = HEAD_INTERNAL_NODE_SIZE + 8;
                for kv in leaf_node.keyvalues.iter() {
                    /* if idx > DEFAULT_MAX_LEAF_ITEMS_NUM as usize{
                        return true;
                    } */
                    threshold_value += kv.key.len() + kv.value.len() + 4;
                }
                (threshold_value as f64) > DEFAULT_MAX_THRESHOLD
            }
            TypedNode::Empty => todo!(),
        }
    }
}
