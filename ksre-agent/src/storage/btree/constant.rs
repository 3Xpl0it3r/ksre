use std::usize;

// 1B nodetype
// 8B pointer to prev node
// 8B pointer to next node
// 8B for key count
pub const HEAD_LEAF_NODE_SIZE: usize = 1 + 8 + 8 + 8;

// 1B for node type
// 8B for key count
// 8B for child count
pub const HEAD_INTERNAL_NODE_SIZE: usize = 1 + 8 + 8;

pub const DEFAULT_PAGE_SIZE: usize = 1024 * 1024;

pub const DEFAULT_META_PN: u64 = 0;
/* pub const DEFAULT_FREELIST_PN: u64 = 1; */

pub const DEFAULT_MAX_THRESHOLD: f64 = 0.90 * DEFAULT_PAGE_SIZE as f64;
pub const DEFAULT_MIN_THRESHOLD: f64 = 0.25 * DEFAULT_PAGE_SIZE as f64;

/* pub const DEFAULT_MAX_KEY_SIZE: usize = 32;
pub const DEFAULT_MAX_VALUE_SIZE: usize = 128; */

/* const DEFAULT_MAX_KV_SIZE: usize = DEFAULT_MAX_KEY_SIZE + DEFAULT_MAX_VALUE_SIZE; */

/* // key_size cost 2B , value size cost 2B
pub const DEFAULT_MAX_LEAF_ITEMS_NUM: f64 = ((DEFAULT_PAGE_SIZE - HEAD_LEAF_NODE_SIZE) as f64)
    .div(DEFAULT_MAX_KV_SIZE as f64 + 4.0)
    .round();


pub const DEFAULT_MAX_INTERNAL_ITEMS_NUM: f64 =
    ((DEFAULT_PAGE_SIZE - HEAD_INTERNAL_NODE_SIZE) as f64).div(DEFAULT_MAX_KEY_SIZE as f64 + 8.0); */
