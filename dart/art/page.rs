
use super::node::{Node4, NODE4_SIZE};
use super::skiplist::{SkipList, Value};

use std::u8;
use std::rc::Rc;
use std::cell::RefCell;
use std::alloc::{GlobalAlloc, System, Layout};
use std::cmp::Ordering;
use std::ptr;
use std::mem;
use std::thread_local;
use std::borrow::BorrowMut;


pub const PAGE_SIZE: u16 = 4 * 1024;
const PAGE_META_SIZE: u16 = 128;

const INDEX_TYPE: u8 = 1;
const DATA_TYPE: u8 = 2;

pub struct IndexPage {
    dirty: bool,
    space: *mut u8,         // page space
    free_space: Vec<u16>,
    free_offset: Vec<u16>,
    free_max: u16,
    index: u32,
}

impl IndexPage {

    pub fn new() -> IndexPage {
        let ly = Layout::from_size_align(PAGE_SIZE as usize, 8).expect("create layout fail");
        unsafe { 
            let raw = System.alloc(ly);
            let mut p = IndexPage{
                dirty: false,
                space: raw,
                free_space: Vec::new(),
                free_offset: Vec::new(),
                free_max: PAGE_SIZE - PAGE_META_SIZE,
                index: 0u32,
            };
            p.free_space.push(PAGE_SIZE - PAGE_META_SIZE);
            p.free_offset.push(PAGE_META_SIZE);
            p.space.write(INDEX_TYPE);
            p
        }
    }

    fn new_empty_page(size: u16) -> IndexPage {
        IndexPage{
            dirty: false,
            space: ptr::null_mut(),
            free_space: Vec::new(),
            free_offset: Vec::new(),
            free_max: size,
            index: 0u32,
        }
    }

    pub fn set_dirty(& mut self, b: bool) {
        self.dirty = b;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    pub fn delete(&mut self) {
        let ly = Layout::from_size_align(PAGE_SIZE as usize, 8).expect("create layout fail");
        unsafe {
            System.dealloc(self.space, ly);
        }
    }

    pub fn new_node4(&mut self) -> Node4 {
        let mut idx = 0;
        for i in 0..self.free_space.len() {
            if self.free_space[i] >= NODE4_SIZE {
                idx = i;
                break;
            }
        }
        let offset = self.free_offset[idx];
        let size = self.free_space[idx];
        unsafe {
            let n = Node4::new(self.space.offset(offset as isize), self.index, offset as u32);
    
            if size > NODE4_SIZE {
                self.free_offset[idx] = offset + NODE4_SIZE;
                self.free_space[idx] = size - NODE4_SIZE;
            } else if size == NODE4_SIZE {
                self.free_offset.remove(idx);
                self.free_space.remove(idx);
            }
            self.free_max = 0;
            for i in 0..self.free_space.len() {
                if self.free_space[i] > self.free_max {
                    self.free_max = self.free_space[i];
                }
            }
            n
        }
    }
}

impl Ord for IndexPage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.free_max.cmp(&other.free_max)
    }
}

impl PartialOrd for IndexPage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for IndexPage {
    fn eq(&self, other: &Self) -> bool {
        self.free_max == other.free_max
    }
}

impl Eq for IndexPage {}

impl Value for IndexPage {
    fn value(&self) -> u16 {
        self.free_max
    }
}

pub static mut IndexPoolInstance: *mut IndexPool = 0 as *mut IndexPool;

pub struct IndexPool {
    pub available_set: SkipList<IndexPage>,
    pub full_set: Vec<IndexPage>,
    pub empty_set: Vec<IndexPage>,
    pub unify_space: Vec<*mut u8>,
}

impl IndexPool {

    pub fn new() {
        let ipl = IndexPool{
            available_set: SkipList::new(32),
            full_set: Vec::new(),
            empty_set: Vec::new(),
            unify_space: Vec::new(),
        };
        unsafe {
            IndexPoolInstance = Box::into_raw(Box::new(ipl));
        }
    }

    pub fn new_page(&mut self) -> Option<IndexPage> {
        let mut p = IndexPage::new();
        let space: *mut u8 = p.space;

        // set page meta
        unsafe {
            // fileno
            space.offset(1).write(0 as u8);
            // offset in file
            let ptr = 10u32.to_be_bytes().as_ptr(); 
            space.offset(2).copy_from(ptr, 4);
            // state
            let sptr = 3u16.to_be_bytes().as_ptr();
            space.offset(6).copy_from(sptr, 2);
            self.unify_space.push(space);
            p.index = self.unify_space.len() as u32;
        }
        Some(p)
    }
   
    pub fn alloc_node4(&mut self) -> Option<Node4> {
        let page = self.find_page(NODE4_SIZE);
        match page {
            Some(mut p) => {
                let n = p.new_node4();
                self.put_page(p);
                Some(n)
            },
            None => None
        }
    }

    pub fn put_page(&mut self, page: IndexPage) {
        if page.free_max > NODE4_SIZE {
            self.available_set.insert(page);
        } else {
            self.full_set.push(page);
        }
    }

    pub fn find_page(&mut self, size: u16) -> Option<IndexPage> {
        // search in available set
        let mut target = IndexPage::new_empty_page(size);
        let res = self.available_set.get(target);
        if res.is_some() {
            return res;
        }

        // search in empty set
        let p = self.empty_set.pop();
        if p.is_some() {
            return res;
        }

        // create new page
        self.new_page()
    }

    pub fn get_space(&self, addr: u32) -> *mut u8 {
        let idx = addr / PAGE_SIZE as u32;
        let offset = addr % PAGE_SIZE as u32;
        if self.unify_space.len() < idx as usize{
            return ptr::null_mut();
        }
        unsafe {
            self.unify_space[idx as usize].offset(offset as isize)
        }
    }
}





pub struct DataPage {
    dirty: bool,
    space: *mut u8,
    free_space: Vec<u16>,
    free_offset: Vec<u16>,
}

pub struct DataPool {
    active_set: Vec<Rc<RefCell<DataPage>>>,
}



















