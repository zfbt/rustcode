extern crate rand;

use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;

use rand::Rng;

const SKIPLIST_MAXLEVEL: i32 = 32;
const SKIPLIST_P: i32 = 0;

pub trait Value {
    fn value(&self) -> u16;
}

pub struct SkipList<T> {
    head: Rc<RefCell<ListNode<T>>>,
    tail: Rc<RefCell<ListNode<T>>>,

    level: i32,
    length: u64,
}

struct ListNode<T> {
    // pointers 
    levels: Vec<Option<Rc<RefCell<ListNode<T>>>>>,

    // backward pointer
    backward: Option<Rc<RefCell<ListNode<T>>>>,

    // current node value
    current: Option<Rc<RefCell<T>>>,
}

// random_level() refer to implement of redis 
fn random_level() -> i32 {
    let mut rng = rand::thread_rng();
    let mut level = 1;

    let mut rn: i32 = rng.gen();
    while rn > SKIPLIST_P && level < SKIPLIST_MAXLEVEL {
        level += 1;
        rn = rng.gen();
    }
    level
}


impl<T> SkipList<T> 
where
    T: Ord + Value
{
    pub fn new(level: i32) -> SkipList<T> {
        let mut lv = level;
        if lv <= 0 || lv > SKIPLIST_MAXLEVEL {
            lv = SKIPLIST_MAXLEVEL;
        }

        let mut h = ListNode{
            levels: Vec::new(),
            backward: None,
            current: None,
        };
        let mut t = ListNode{
            levels: Vec::new(),
            backward: None,
            current: None,
        };
        for i in 0..lv {
            t.levels.push(None);
        }
        let ptr = Rc::new(RefCell::new(t));
        for i in 0..lv {
            h.levels.push(Some(ptr.clone()));
        }

        let sl = SkipList{
            head: Rc::new(RefCell::new(h)),
            tail: ptr.clone(),
            level: lv,
            length: 0 as u64,
        };
        sl
    }

    pub fn len(&self) -> u64 {
        self.length
    }

    pub fn insert(&mut self, ele: T) {
        //println!("calculate random level");
        let mut level = random_level();
        //println!("calculate random level result: {}", level);
        let mut node = ListNode{
            levels: Vec::new(),
            backward: Some(self.head.clone()),
            current: None,
        };
        node.levels.resize_with(level as usize, || { None });

        let mut prev_nodes: Vec<Option<Rc<RefCell<ListNode<T>>>>> = Vec::new();
        let mut curr: Rc<RefCell<ListNode<T>>> = self.head.clone();
        prev_nodes.resize_with(level as usize, || { None });

        let mut level_idx = self.level - 1;
        while level_idx >= 0 {
            //println!("check level: {}", level_idx);
            loop {
                let next = curr.borrow_mut().levels[level_idx as usize].as_ref().unwrap().clone();
                // next is tail node
                if next.borrow_mut().current.is_none() {
                    //println!("level: {} end with none", level_idx);
                    if level_idx <= level - 1 {
                        prev_nodes[level_idx as usize] = Some(curr.clone());
                    }
                    break;
                }
                // go down
                let res = ele.cmp(&next.borrow_mut().current.as_ref().unwrap().borrow());
                match res {
                    Ordering::Less | Ordering::Equal => {
                        if level_idx <= level -1 {
                            prev_nodes[level_idx as usize] = Some(curr.clone());
                        }
                        break;
                    },
                    _ => {
                        curr = next.clone();
                    },
                }
            }
            level_idx -= 1;
        }

        // insert to list
        node.current = Some(Rc::new(RefCell::new(ele)));
        let mut curr = Rc::new(RefCell::new(node));
        for i in 0..level {
            let prev = prev_nodes[i as usize].as_ref().unwrap().clone();
            let next = prev.borrow_mut().levels[i as usize].as_ref().unwrap().clone();

            prev.borrow_mut().levels[i as usize] = Some(curr.clone());
            curr.borrow_mut().levels[i as usize] = Some(next.clone());
            if i == 0 {
                curr.borrow_mut().backward = Some(prev.clone());
                next.borrow_mut().backward = Some(curr.clone());
            }
        }
        self.length += 1;
    }

    // Try to find a node N who makes ele.Eq(N) true,
    // if no such node exist in skiplist, return the smallest
    // node who is large than ele.
    pub fn search(&self, ele: T) -> Option<Rc<RefCell<T>>> {
        let mut curr: Rc<RefCell<ListNode<T>>> = self.head.clone();
        let mut level_idx = self.level - 1;
        let mut found = false;
        while level_idx >= 0 {
            found = false;
            loop {
                let next = curr.borrow_mut().levels[level_idx as usize].as_ref().unwrap().clone();
                // next is tail node
                if next.borrow_mut().current.is_none() {
                    break;
                }
                // go down
                let res = ele.cmp(&next.borrow_mut().current.as_ref().unwrap().borrow());
                match res {
                    Ordering::Less => {
                        break;
                    },
                    Ordering::Equal => {
                        found = true;
                        curr = next.clone();
                        break;
                    }
                    _ => {
                        curr = next.clone();
                    },
                }
            }
            if found {
                break;
            }
            level_idx -= 1;
        }

        let next = curr.borrow_mut().levels[0].as_ref().unwrap().clone();
        if found {
            Some(curr.borrow_mut().current.as_ref().unwrap().clone())
        } else {
            match next.borrow_mut().current {
                Some(ref rc) => Some(rc.clone()),
                None => None, 
            }
        }
    }

    // Get is much like search except that it will delete the node
    // from skiplist. returned type is not pointer
    pub fn get(&mut self, ele: T) -> Option<T> {
        let mut curr: Rc<RefCell<ListNode<T>>> = self.head.clone();
        let mut prev_nodes: Vec<Option<Rc<RefCell<ListNode<T>>>>> = Vec::new();
        let mut level_idx = self.level - 1;
        prev_nodes.resize_with(self.level as usize, || { None });
        while level_idx >= 0 {
            loop {
                let next = curr.borrow_mut().levels[level_idx as usize].as_ref().unwrap().clone();
                // next is tail node
                if next.borrow_mut().current.is_none() {
                    prev_nodes[level_idx as usize] = Some(curr.clone());
                    break;
                }
                // go down
                let res = ele.cmp(&next.borrow_mut().current.as_ref().unwrap().borrow());
                match res {
                    Ordering::Less | Ordering::Equal => {
                        prev_nodes[level_idx as usize] = Some(curr.clone());
                        break;
                    },
                    _ => {
                        curr = next.clone();
                    },
                }
            }
            level_idx -= 1;
        }

        // delete from list
        let curr = curr.borrow_mut().levels[0 as usize].as_ref().unwrap().clone();
        if curr.borrow_mut().current.is_none() {
            return None;
        }
        let level = curr.borrow_mut().levels.len();
        for i in 0..level {
            let prev = prev_nodes[i as usize].as_ref().unwrap().clone();
            let next = curr.borrow_mut().levels[i as usize].as_ref().unwrap().clone();

            prev.borrow_mut().levels[i as usize] = Some(next.clone());
            curr.borrow_mut().levels[i as usize] = None;
            if i == 0 {
                curr.borrow_mut().backward = None;
                next.borrow_mut().backward = Some(prev.clone());
            }
        }
        self.length -= 1;

        match Rc::try_unwrap(curr) {
            Ok(node) => {
                let value = node.into_inner().current.expect("current is none");
                match Rc::try_unwrap(value) {
                    Ok(v) => Some(v.into_inner()),
                    Err(_) => None,
                }
            },
            Err(_) => None,
        }
    }

    pub fn print(&self, level: usize) {
        println!("============ level {} nodes: =============", level);
        let mut curr = self.head.borrow_mut().levels[level].as_ref().unwrap().clone();
        loop {
            if curr.borrow_mut().current.is_none() {
                break;
            }
            let v = curr.borrow_mut().current.as_ref().unwrap().borrow_mut().value();
            println!("value: {}", v);
            let next = curr.borrow_mut().levels[level].as_ref().unwrap().clone();
            curr = next.clone();
        }
    }
}

















