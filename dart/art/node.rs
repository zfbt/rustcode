use super::page::{IndexPool, IndexPoolInstance, PAGE_SIZE};
use std::slice;
use std::u32;
use std::rc::Rc;
use std::cell::RefCell;

pub const NODE4_SIZE: u16 = 64;
pub const NODE16_SIZE: u16 = 192;
pub const NODE48_SIZE: u16 = 576;
pub const NODE256_SIZE: u16 = 1600;

// node state
const ACTIVE: u8 = 1;
const DELETED: u8 = 2;

// free length
const NODE4_FREE_LEN: u16 = 28;
const NODE16_FREE_LEN: u16 = 72;
const NODE48_FREE_LEN: u16 = 24;
const NODE256_FREE_LEN: u16 = 56;

// value type
const NONE: u8 = 0;
const ADDR: u8 = 1;
const DATA: u8 = 2;

// Pointer size 
const POINTER_SIZE: u16 = 6;
const NULL_PTR: u8 = 0;

// Node format
const EDGE_LEN_OFFSET: isize = 1;
const PARENT_ADDR_OFFSET: isize = 2;
const CHILD_KEY_OFFSET: isize = 8;
const NODE4_EDGE_OFFSET: isize = 36;


// Record format
const LENGH_OFFSET: isize = 1;

enum NodeOpRes{
    Succ,
    NotEnoughSpace,
    ChildExists,
}

enum NodeType {
    N4,
    N16,
    N48,
    N256,
}

pub trait ArtNode {
    fn shrink(&mut self) -> NodeOpRes {
        NodeOpRes::Succ
    }
    fn expand(&mut self) -> NodeOpRes {
        NodeOpRes::Succ
    }

    fn need_expand(&self) -> bool {
        false
    }
    fn need_shrink(&self) -> bool {
        false
    }

    fn set_edge(&mut self, edge: &[u8]) -> NodeOpRes {
        NodeOpRes::Succ
    }
    fn get_edge(&self) -> &[u8];

    fn get_child(&self, key: u8) -> Option<Box<dyn ArtNode>>;
    fn set_child(&self, key: u8, node: Box<dyn ArtNode>) -> NodeOpRes;

    fn get_child_distance(&self, key: u8) -> u8 {
        0u8
    }
    fn get_position(&self) -> u32 {
        0u32
    }
}

pub struct Art {
    root: Rc<RefCell<dyn ArtNode>>,
    size: u32,
}

impl Art {
    pub fn new()-> Art {
        unsafe {
            let mut node4 = (*IndexPoolInstance).alloc_node4().expect("alloc node failed");
            let art = Art{
                root: Rc::new(RefCell::new(node4)),
                size: 1u32,
            };
            art
        }
    }

    pub fn insert(&mut self, key: String, value: String) -> NodeOpRes {
        let cs = key.as_bytes(); 
        let cursor = 0usize;
        let curr = root.clone();

        // nothing is ready
        loop {
            let c: u8 = cs[cursor];
            let res = curr.borrow().get_child(c);
            // create a new child node
            if res.is_none() {
                return self.create_child_node(curr.clone(), key, value);
            }
            // already exist a child node whose edge start with c
            let cnode = res.unwrap();
            let edge = cnode.get_edge();
            let key_len = cs.len() - cursor;
            if edge.len() < key_len {
                let prefix_idx = find_common_prefix(edge, &cs[cursor..]);
                // edge is prefix of key
                if prefix_idx == edge.len() {
                    curr = Rc::new(RefCell::new(*cnode));
                    cursor = prefix_idx;
                    continue;
                }
                // edge is not prefix, create a middle node with common prefix
                let mut middle_node = (*IndexPoolInstance).alloc_node4().expect("alloc node failed");
                let mut new_node = (*IndexPoolInstance).alloc_node4().expect("alloc node failed");
                // new node is for key
                new_node.set_edge(cs[cursor+prefix_idx..]);
                new_node.set_value(value);
                
                // middle node's edge is common prefix
                middle_node.set_edge(&edge[prefix_idx..]);
                middle_node.set_child(cs[cursor], Box::new(new_node));
                middle_node.set_child(cs[cursor+prefix_idx..], Box::new(cnode));
                
                // curr node point to middle node
                curr.set_child(Box::new(middle_node));
                break;
            }
            // key exist, rewrite value
            if edge.len() == key_len && slice_equal(edge, &cs[cursor..]) {
                cnode.set_value(value);
                break;
            }

            if edge.len() > key_len {

            }

        }
    }
}



pub struct Node4 {
    space: *mut u8,
    position: u32,
}

impl Node4 {

    pub unsafe fn new(buf: *mut u8, idx: u32, offset: u32) -> Node4 {
        let mut n = Node4{
            space: buf,
            position: make_pointer_addr(idx, offset),
        };
        unsafe {
            n.space.write(ACTIVE);
            n
        }
    }
    
    fn get_valid_child_count(&self) -> u8 {
        let mut valid_ptr_cnt = 0u8;
        unsafe {
            let ptrs = self.space.offset(CHILD_KEY_OFFSET + 4);
            for i in 0..4 {
                if ptrs.offset(i*6).read() != NULL_PTR {
                    valid_ptr_cnt += 1;
                }
            }
        }
        valid_ptr_cnt
    }

    fn get_child_idx(&self, key: u8) -> isize {
        let mut idx = -1isize;
        unsafe {
            let children = self.space.offset(CHILD_KEY_OFFSET);
            for i in 0..4 {
                if children.offset(i).read() == key {
                    idx = i;
                }
            }
        }
        idx
    }
}

impl ArtNode for Node4 {

    // Node4
    fn set_edge(&mut self, edge: &[u8]) -> NodeOpRes {
        if edge.len() > (NODE4_FREE_LEN - POINTER_SIZE - 1) as usize {
            return NodeOpRes::NotEnoughSpace;
        }
        let edge_len = edge.len() as u8;
        unsafe {
            self.space.offset(EDGE_LEN_OFFSET).write(edge_len);
            self.space.offset(NODE4_EDGE_OFFSET).copy_from(edge.as_ptr(), edge_len as usize);
            NodeOpRes::Succ
        }
    }

    // Node4
    fn get_edge(&self) -> &[u8] {
        let edge: &[u8] = &[];
        unsafe {
            let edge_len = self.space.offset(1).read();
            if self.space.offset(1).read() > 0 {
                return slice:: from_raw_parts(self.space.offset(NODE4_FREE_LEN as isize), edge_len as usize);
            }
        }
        edge
    }

    // Node4
    fn get_child(&self, key: u8) -> Option<Box<dyn ArtNode>> {
        let idx = self.get_child_idx(key);
        if idx == -1 {
            return None;
        }
        unsafe {
            // get child pointer
            let ptr_space = self.space.offset(CHILD_KEY_OFFSET + 4 + idx*6);
            // null pointer
            if ptr_space.read() == NULL_PTR {
                return None;
            }
            let child_addr = get_child_space(ptr_space);

            let info = ptr_space.offset(NODE_INFO_OFFSET).read();
            let child_space = (*IndexPoolInstance).get_space(child_addr);
            match get_type_from_info(info) {
                NodeType::N4 => Some(Box::new(Node4::new(child_space, child_addr / PAGE_SIZE as u32, child_addr % PAGE_SIZE as u32))),
                NodeType::N16 => Some(Box::new(Node4::new(child_space, child_addr / PAGE_SIZE as u32, child_addr % PAGE_SIZE as u32))),
                NodeType::N48 => Some(Box::new(Node4::new(child_space, child_addr / PAGE_SIZE as u32, child_addr % PAGE_SIZE as u32))),
                NodeType::N256 => Some(Box::new(Node4::new(child_space, child_addr / PAGE_SIZE as u32, child_addr % PAGE_SIZE as u32))),
            }
        }
    }

    // Node4
    fn get_child_distance(&self, key: u8) -> u8 {
        let idx = self.get_child_idx(key);
        if idx == -1 {
            return 0;
        }
        unsafe {
            // get child pointer
            let ptr_space = self.space.offset(CHILD_KEY_OFFSET + 4 + idx*6);
            // null pointer
            if ptr_space.read() == NULL_PTR {
                return 0;
            }
            let info = ptr_space.offset(NODE_INFO_OFFSET).read();
            get_distance_from_info(info)
        }
    }
    
    // Node4
    fn set_child(&self, key: u8, node: Box<dyn ArtNode>) -> NodeOpRes {
        let idx = self.get_child_idx(key);
       
        unsafe {
            let child_space = self.space.offset(CHILD_KEY_OFFSET);
            if idx >= 0 {
                if child_space.offset(4 + idx*6).read() != NULL_PTR {
                    return NodeOpRes::ChildExists;
                }
            }
            for i in 0..4 {
                if child_space.offset(4 + i*6).read() == NULL_PTR {
                    // write key
                    child_space.offset(i).write(key);
                    // write pointer
                    let distance = node.as_ref().unwrap().get_edge().len();
                    let addr = node.get_position();
                    child_space.offset(4 + i*6).write(1u8); // fixed fileno
                    child_space.offset(4 + i*6 + 1).write(distance as u8);
                    let ptr = addr.to_be_bytes().as_ptr(); 
                    child_space.offset(4 + i*6 + 2).copy_from(ptr, 4);
                    return NodeOpRes::Succ;
                }
            }
        }
        NodeOpRes::NotEnoughSpace
    }

    // Node4
    fn need_expand(&self) -> bool {
        let valid_cnt = self.get_valid_child_count();
        if valid_cnt == 4 {
            return true;
        }
        false
    }

    // Node4
    fn need_shrink(&self) -> bool {
        false
    }
    
    // Node4
    fn expand(&mut self) -> NodeOpRes {
        NodeOpRes::Succ
    }

    // Node4
    fn shrink(&mut self) -> NodeOpRes {
        NodeOpRes::Succ
    }
    
    fn get_position(&self) -> u32 {
        self.position
    }
}




// Pointer format
const NODE_INFO_OFFSET: isize = 1;
const ADDR_OFFSET: isize = 2;
        
fn get_distance_from_info(info: u8) -> u8 {
    info & 0x3F
}

fn get_type_from_info(info: u8) -> NodeType {
    match info & 0xC0 >> 6 {
        0 => NodeType::N4,
        1 => NodeType::N16,
        2 => NodeType::N48,
        3 => NodeType::N256,
        _ => NodeType::N4,
    }
}

fn gen_node_info(distance: u8, tp: u8) -> u8 {
    tp << 6 | distance
}

fn get_child_space(space: *const u8) -> u32 {
    let mut bytes = [0, 0, 0, 0];
    unsafe {
        bytes[0] = space.offset(ADDR_OFFSET).read();
        bytes[1] = space.offset(ADDR_OFFSET + 1).read();
        bytes[2] = space.offset(ADDR_OFFSET + 2).read();
        bytes[3] = space.offset(ADDR_OFFSET + 3).read();
    }
    u32::from_be_bytes(bytes)
}

fn make_pointer_addr(idx: u32, offset: u32) -> u32 {
    (idx << 12) | offset
}







