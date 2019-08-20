use std::u8;
use std::rc::Rc;
use std::cell::RefCell;

pub trait ArtNode {
    fn shrink(&mut self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        None 
    }
    fn expand(&mut self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        None 
    }
    
    fn get_edge(&self) -> u8 {
        0 as u8
    }

    fn need_expand(&self) -> bool {
        false
    }
    fn need_shrink(&self) -> bool {
        false
    }

    fn set_value(&mut self, _val: String) {}
    fn get_values(&self) -> Option<Vec<String>> {
        None
    }
    fn del_values(&mut self) {}

    fn set_child(&mut self, key: u8, ch: Option<Rc<RefCell<dyn ArtNode>>>) -> bool;
    fn get_child(&mut self, key: u8) -> Option<Rc<RefCell<dyn ArtNode>>>;

    fn del_child(&mut self, key: u8);

    fn get_children(&mut self) -> Vec<Option<Rc<RefCell<dyn ArtNode>>>> {
        Vec::new()
    }

    fn empty(&self) -> bool {
        false
    }

    fn set_parent(&mut self, _pare: Option<Rc<RefCell<dyn ArtNode>>>) {}
    fn get_parent(&self) -> Option<Rc<RefCell<dyn ArtNode>>> { None }
}


pub struct Art {
    sentinel: Rc<RefCell<dyn ArtNode>>,
    root: Option<Rc<RefCell<dyn ArtNode>>>,
    size: usize,
}

impl Art {

    pub fn new() -> Art {
        let mut art = Art {
            sentinel: Rc::new(RefCell::new(Sentinel::new())),
            root: None,
            size: 1usize,
        };
        let mut node4 = Node4::new(0);
        node4.parent = Some(art.sentinel.clone());
        let rc = Rc::new(RefCell::new(node4)); 
        art.sentinel.borrow_mut().set_child(0, Some(rc.clone()));
        art.root = Some(rc.clone());
        art
    }

    pub fn insert(&mut self, key: String, val: String) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let cs = key.as_bytes();
        if self.root.is_none() {
            println!("root can't be None");
            return None;
        }

        let mut curr_node = self.root.as_ref().unwrap().clone();
        for c in cs.iter() {
            let cn = curr_node.borrow_mut().get_child(*c as u8);
            // child node exists
            if cn.is_some() {
                curr_node = cn.as_ref().unwrap().clone();
                continue;
            }
            // child node not exists
            let mut node4 = Node4::new(*c as u8);
            node4.parent = Some(curr_node.clone());
            let rc = Rc::new(RefCell::new(node4)); 

            // set a new child node for c
            if curr_node.borrow_mut().set_child(*c as u8, Some(rc.clone())) {
                curr_node = rc.clone();
                self.size += 1;
                continue;
            }

            // set child node failed, maybe need expand
            if curr_node.borrow_mut().need_expand() {
                let mut candi = curr_node.borrow_mut().expand();
                if candi.is_none() {
                    println!("expand node error");
                    return None;
                }
                if !candi.as_mut().unwrap().borrow_mut().set_child(*c as u8, Some(rc.clone())) {
                    println!("still set child failed after expand");
                    return None;
                }
                // set parent's child to new node
                let k = curr_node.borrow_mut().get_edge();
                let p = curr_node.borrow_mut().get_parent();
                match p {
                    Some(ref prc) => prc.borrow_mut().set_child(k, candi),
                    None => {
                        println!("Parent node can't be None");
                        return None;
                    },
                };
                self.size += 1;
                curr_node = rc.clone();
            } else {
                return None;
            }
        }
        curr_node.borrow_mut().set_value(val);
        Some(curr_node.clone())
    }

    pub fn search(&self, key: &String) -> Option<Vec<String>> {
        let cs = key.as_bytes();
        if self.root.is_none() {
            println!("root can't be None");
            return None;
        }

        let mut curr_node = self.root.as_ref().unwrap().clone();
        for c in cs.iter() {
            let cn = curr_node.borrow_mut().get_child(*c as u8);
            if cn.is_none() {
                println!("Search: no such key: {}", key);
                return None;
            }
            curr_node = cn.as_ref().unwrap().clone();
        }

        let r = curr_node.borrow_mut().get_values();
        r
    }

    pub fn delete(&mut self, key: &String) -> bool {
        let cs = key.as_bytes();
        let mut curr_node = self.root.as_ref().unwrap().clone();

        for c in cs.iter() {
            let cn = curr_node.borrow_mut().get_child(*c as u8);
            if cn.is_none() {
                println!("Delete: no such key: {}", key);
                return false;
            }
            curr_node = cn.as_ref().unwrap().clone();
        }

        // key not exist
        if curr_node.borrow_mut().get_values().as_ref().unwrap().len() == 0 {
            println!("Delete: no values of key: {}", key);
            return false;
        }

        // delete values
        curr_node.borrow_mut().del_values();

        // if no sub keys exist, curr_node should be deleted,
        // also some node may need shrink
        loop {
            // empty node
            if curr_node.borrow_mut().empty() {
                if Rc::ptr_eq(self.root.as_ref().unwrap(), &curr_node) {
                    break;
                }
                let key = curr_node.borrow_mut().get_edge();
                let p = curr_node.borrow_mut().get_parent();
                match p {
                    Some(ref rc) => {
                        println!("delete node: {}", key as char);
                        rc.borrow_mut().del_child(key);
                        self.size -= 1;
                    }
                    None => {
                        println!("parent can't be None, current key: {}", key as char);
                        return false;
                    },
                }
                curr_node = p.as_ref().unwrap().clone();
                continue
            }
            // not empty, but need shrink
            if curr_node.borrow_mut().need_shrink() {
                let candi = curr_node.borrow_mut().shrink();
                if candi.is_none() {
                    println!("shrink node failed");
                    return false;
                }
                // set parent's child to new node
                let k = curr_node.borrow_mut().get_edge();
                let p = curr_node.borrow_mut().get_parent();
                match p {
                    Some(ref prc) => prc.borrow_mut().set_child(k, candi),
                    None => {
                        println!("Parent node can't be None");
                        return false;
                    },
                };
            }
            // nothing to do
            break;
        }
        true
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}

pub struct Node4 {
    edge: u8,
    keys: [u8; 4],
    children: Vec<Option<Rc<RefCell<dyn ArtNode>>>>,
    parent: Option<Rc<RefCell<dyn ArtNode>>>,
    values: Vec<String>,
}

pub struct Node16 {
    edge: u8,
    keys: [u8; 16],
    children: Vec<Option<Rc<RefCell<dyn ArtNode>>>>,
    parent: Option<Rc<RefCell<dyn ArtNode>>>,
    values: Vec<String>,
}

pub struct Node48 {
    edge: u8,
    keys: [i8; 256],
    children: Vec<Option<Rc<RefCell<dyn ArtNode>>>>,
    parent: Option<Rc<RefCell<dyn ArtNode>>>,
    values: Vec<String>,
}

pub struct Node256 {
    edge: u8,
    children: Vec<Option<Rc<RefCell<dyn ArtNode>>>>,
    parent: Option<Rc<RefCell<dyn ArtNode>>>,
    values: Vec<String>,
}

impl Node4 {
    pub fn new(edge: u8) -> Node4 {
        let mut node = Node4 {
            edge: edge,
            keys: [0; 4],
            children: Vec::new(),
            parent: None,
            values: Vec::new(),
        };
        node.children.resize_with(4, || { None });
        node
    }
}

impl Node16 {
    pub fn new(edge: u8) -> Node16 {
        let mut node = Node16 {
            edge: edge,
            keys: [0; 16],
            children: Vec::new(),
            parent: None,
            values: Vec::new(),
        };
        node.children.resize_with(16, || { None });
        node
    }
}

impl Node48 {
    pub fn new(edge: u8) -> Node48 {
        let node = Node48 {
            edge: edge,
            keys: [-1; 256],
            children: Vec::with_capacity(48),
            parent: None,
            values: Vec::new(),
        };
        node
    }
}

impl Node256 {
    pub fn new(edge: u8) -> Node256 {
        let mut node = Node256 {
            edge: edge,
            children: Vec::new(),
            parent: None,
            values: Vec::new(),
        };
        node.children.resize_with(256, || { None });
        node
    }
}

impl ArtNode for Node4 {

    // Node4
    fn get_child(&mut self, key: u8) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let mut idx: u8 = u8::MAX;
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                idx = i as u8;
                break;
            }
        }
        if idx == u8::MAX {
            return None; 
        }
        match self.children[idx as usize] {
            Some(ref rc) => Some(rc.clone()),
            None => None,
        }
    }

    // Node4
    // ch's parent node should be set already
    fn set_child(&mut self, key: u8, ch: Option<Rc<RefCell<dyn ArtNode>>>) -> bool {
        for i in 0..4 {
            if self.keys[i] == key {
                match self.children[i] {
                    Some(ref rc) => {
                        rc.borrow_mut().set_parent(None);
                    },
                    None => {},
                }
                self.children[i] = ch;
                self.keys[i] = key;
                return true;
            }
        }
        
        let mut idx: usize = 0;
        let mut has_slot = false;
        for i in 0..4 {
            if self.children[i].is_none() {
                idx = i;
                has_slot = true;
                self.children[i] = ch;
                break;
            }
        }
        if has_slot == false {
            return false;
        }
        self.keys[idx] = key;
        true
    }
    
    // Node4
    fn del_child(&mut self, key: u8) {
        for i in 0..4 {
            if self.keys[i] == key {
                match self.children[i] {
                    Some(ref rc) => {
                        rc.borrow_mut().set_parent(None);
                    },
                    None => {},
                }
                self.children[i] = None;
            }
        }
    }

    // Node4
    fn get_children(&mut self) -> Vec<Option<Rc<RefCell<dyn ArtNode>>>> {
        let mut v: Vec<Option<Rc<RefCell<dyn ArtNode>>>> = Vec::new();
        v.clone_from(&self.children);
        v
    }
    
    // Node4
    fn empty(&self) -> bool {
        for i in 0..4 {
            if self.children[i].is_some() {
                return false;
            }
        }
        true
    }

    // Node4
    fn expand(&mut self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let mut node16 = Node16::new(self.edge);
        
        // copy values to new node
        node16.values.clone_from(&mut self.values);

        // copy children to new node
        for i in 0..4 {
            match self.children[i] {
                Some(ref rc) => node16.children[i] = Some(rc.clone()),
                None => {},
            }
            node16.keys[i] = self.keys[i];
        }
       
        // set parent for new node
        match self.parent {
            Some(ref rc) => node16.parent = Some(rc.clone()),
            None => {
                println!("Parent node can't be None");
                return None;
            },
        }
        let rc = Rc::new(RefCell::new(node16));

        // set children's parent to new node
        let chs = rc.borrow_mut().get_children();
        for cn in chs.iter() {
            match *cn {
                Some(ref crc) => crc.borrow_mut().set_parent(Some(rc.clone())),
                None => {},
            }
        }
        Some(rc.clone())
    }
    
    // Node4
    fn get_edge(&self) -> u8 {
        self.edge
    }
    
    // Node4
    fn need_expand(&self) -> bool {
        let mut full = true;
        for i in 0..4 {
            if self.children[i].is_none() {
                full = false;
                break;
            }
        }
        full
    }
    
    // Node4
    fn need_shrink(&self) -> bool {
        false
    }
    
    // Node4
    fn set_value(&mut self, val: String) {
        self.values.push(val);
    }
    
    // Node4
    fn get_values(&self) -> Option<Vec<String>> {
        let mut r: Vec<String> = Vec::new();
        r.clone_from(&self.values);
        Some(r)
    }
    
    // Node4
    fn del_values(&mut self) {
        self.values.clear();
    }
    
    // Node4
    fn set_parent(&mut self, pare: Option<Rc<RefCell<dyn ArtNode>>>) {
        self.parent = pare;
    }
    
    // Node4
    fn get_parent(&self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        match self.parent {
            Some(ref rc) => return Some(rc.clone()),
            None => return None,
        }
    }
}

impl ArtNode for Node16 {

    // Node16
    fn get_child(&mut self, key: u8) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let mut idx: u8 = u8::MAX;
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                idx = i as u8;
                break;
            }
        }
        if idx == u8::MAX {
            return None; 
        }
        match self.children[idx as usize] {
            Some(ref rc) => Some(rc.clone()),
            None => None,
        }
    }
    
    // Node16
    fn set_child(&mut self, key: u8, ch: Option<Rc<RefCell<dyn ArtNode>>>) -> bool {
        for i in 0..16 {
            if self.keys[i] == key {
                match self.children[i] {
                    Some(ref rc) => {
                        rc.borrow_mut().set_parent(None);
                    },
                    None => {},
                }
                self.children[i] = ch;
                self.keys[i] = key;
                return true;
            }
        }
        
        let mut idx: usize = 0;
        let mut has_slot = false;
        for i in 0..16 {
            if self.children[i].is_none() {
                idx = i;
                has_slot = true;
                self.children[i] = ch;
                break;
            }
        }
        if has_slot == false {
            return false;
        }
        self.keys[idx] = key;
        true
    }
    
    // Node16
    fn del_child(&mut self, key: u8) {
        for i in 0..16 {
            if self.keys[i] == key {
                match self.children[i] {
                    Some(ref rc) => {
                        rc.borrow_mut().set_parent(None);
                    },
                    None => {},
                }
                self.children[i] = None;
            }
        }
    }
    
    // Node16
    fn empty(&self) -> bool {
        for i in 0..16 {
            if self.children[i].is_some() {
                return false;
            }
        }
        true 
    }
    
    // Node16
    fn get_children(&mut self) -> Vec<Option<Rc<RefCell<dyn ArtNode>>>> {
        let mut v: Vec<Option<Rc<RefCell<dyn ArtNode>>>> = Vec::new();
        v.clone_from(&self.children);
        v
    }

    // Node16
    fn shrink(&mut self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let mut node4 = Node4::new(self.edge);
        let mut csize = 0usize;
        
        // copy values to new node
        node4.values.clone_from(&mut self.values);
        
        // copy children to new node
        for i in 0..16 {
            match self.children[i] {
                Some(ref rc) => {
                    node4.children[csize] = Some(rc.clone());
                    node4.keys[csize] = self.keys[i];
                    csize += 1;
                },
                None => {},
            }
            if csize >= 4 {
                break;
            }
        }

        // set parent for new node
        match self.parent {
            Some(ref rc) => node4.parent = Some(rc.clone()),
            None => {
                println!("Parent node can't be None");
                return None;
            }
        }
        let rc = Rc::new(RefCell::new(node4));
        
        // set children's parent to new node
        let chs = rc.borrow_mut().get_children();
        for cn in chs.iter() {
            match *cn {
                Some(ref crc) => crc.borrow_mut().set_parent(Some(rc.clone())),
                None => {},
            }
        }
        Some(rc.clone())
    }

    // Node16
    fn expand(&mut self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let mut node48 = Node48::new(self.edge);
        
        // copy values to new node
        node48.values.clone_from(&mut self.values);

        // copy children to new node
        for i in 0..16 {
            match self.children[i] {
                Some(ref rc) => {
                    let key = rc.borrow().get_edge();
                    node48.children.push(Some(rc.clone()));
                    node48.keys[key as usize] = (node48.children.len()-1) as i8;
                },
                None => {},
            }
        }
       
        // set parent for new node
        match self.parent {
            Some(ref rc) => node48.parent = Some(rc.clone()),
            None => {
                println!("Parent node can't be None");
                return None;
            }
        }
        let rc = Rc::new(RefCell::new(node48));
      
        // set children's parent to new node
        let chs = rc.borrow_mut().get_children();
        for cn in chs.iter() {
            match *cn {
                Some(ref crc) => crc.borrow_mut().set_parent(Some(rc.clone())),
                None => {},
            }
        }
        Some(rc.clone())
    }
    
    // Node16
    fn get_edge(&self) -> u8 {
        self.edge
    }
    
    // Node16
    fn need_expand(&self) -> bool {
        let mut full = true; 
        for i in 0..16 {
            if self.children[i].is_none() {
                full = false;
                break;
            }
        }
        full
    }
    
    // Node16
    fn need_shrink(&self) -> bool {
        let mut count = 0i32;
        for i in 0..16 {
            if self.children[i].is_some() {
                count += 1;
            }
        }
        if count < 3 {
            return true;
        }
        false
    }
    
    // Node16
    fn set_value(&mut self, val: String) {
        self.values.push(val);
    }
    
    // Node16
    fn get_values(&self) -> Option<Vec<String>> {
        let mut r: Vec<String> = Vec::new();
        r.clone_from(&self.values);
        Some(r)
    }
    
    // Node16
    fn del_values(&mut self) {
        self.values.clear();
    }
    
    // Node16
    fn set_parent(&mut self, pare: Option<Rc<RefCell<dyn ArtNode>>>) {
        self.parent = pare;
    }
    
    // Node16
    fn get_parent(&self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        match self.parent {
            Some(ref rc) => return Some(rc.clone()),
            None => return None,
        }
    }
}

impl ArtNode for Node48 {
    
    // Node48
    fn set_child(&mut self, key: u8, ch: Option<Rc<RefCell<dyn ArtNode>>>) -> bool {
        let mut idx = self.keys[key as usize];
        if idx >= 0 {
            self.children[idx as usize] = ch;
            return true;
        }

        for i in 0..48 {
            if self.children[i].is_none() {
                idx = i as i8;
                self.children[i] = ch;
                break;
            }
        }
        // no more place to insert node
        if idx < 0 {
            return false;
        } 
        self.keys[key as usize] = idx; 
        true
    }

    // Node48
    fn del_child(&mut self, key: u8) {
        let idx = self.keys[key as usize];
        if idx >= 0 {
            match self.children[idx as usize] {
                Some(ref rc) => rc.borrow_mut().set_parent(None),
                None => {},
            }
            self.children[idx as usize] = None;
            self.keys[key as usize] = -1;
        }
    }

    // Node48
    fn get_child(&mut self, key: u8) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let idx = self.keys[key as usize];
        if idx < 0 {
            return None;
        }
        match self.children[idx as usize] {
            Some(ref rc) => {
                // wrong pointer
                if rc.borrow().get_edge() != key {
                    self.keys[key as usize] = -1;
                    return None;
                }
                return Some(rc.clone());
            },
            // cleaning index
            None => self.keys[key as usize] = -1,
        }
        None
    }
    
    // Node48
    fn empty(&self) -> bool {
        for i in 0..48 {
            if self.keys[i] >= 0 {
                return false;
            }
        }
        true 
    }
    
    // Node48
    fn get_children(&mut self) -> Vec<Option<Rc<RefCell<dyn ArtNode>>>> {
        let mut v: Vec<Option<Rc<RefCell<dyn ArtNode>>>> = Vec::new();
        v.clone_from(&self.children);
        v
    }

    // Node48
    fn shrink(&mut self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let mut node16 = Node16::new(self.edge);
        let mut csize = 0usize;
       
        // copy values to new node
        node16.values.clone_from(&mut self.values);
        
        // copy children to new node
        for i in 0..48 {
            match self.children[i] {
                Some(ref rc) => {
                    node16.children[csize] = Some(rc.clone());
                    node16.keys[csize] = rc.borrow().get_edge();
                    csize += 1;
                },
                None => {},
            }
            if csize >= 16 {
                break;
            }
        }

        // set parent for new node
        match self.parent {
            Some(ref rc) => node16.parent = Some(rc.clone()),
            None => {
                println!("Parent node can't be None");
                return None;
            }
        }
        let rc = Rc::new(RefCell::new(node16));
        
        // set children's parent to new node
        let chs = rc.borrow_mut().get_children();
        for cn in chs.iter() {
            match *cn {
                Some(ref crc) => crc.borrow_mut().set_parent(Some(rc.clone())),
                None => {},
            }
        }
        Some(rc.clone())
    }
    
    // Node48
    fn expand(&mut self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let mut node256 = Node256::new(self.edge);
        
        // copy values to new node
        node256.values.clone_from(&mut self.values);
        
        // copy children to new node
        for i in 0..48 {
            match self.children[i] {
                Some(ref rc) => {
                    let idx = rc.borrow().get_edge();
                    node256.children[idx as usize] = Some(rc.clone());
                },
                None => {},
            }
        }
        
        // set parent for new node
        match self.parent {
            Some(ref rc) => node256.parent = Some(rc.clone()),
            None => {
                println!("Parent node can't be None");
                return None;
            }
        }
        let rc = Rc::new(RefCell::new(node256));
        
        // set children's parent to new node
        let chs = rc.borrow_mut().get_children();
        for cn in chs.iter() {
            match *cn {
                Some(ref crc) => crc.borrow_mut().set_parent(Some(rc.clone())),
                None => {},
            }
        }
        Some(rc.clone())
    }

    // Node48
    fn get_edge(&self) -> u8 {
        self.edge
    }
    
    // Node48
    fn need_expand(&self) -> bool {
        let mut full = true; 
        for i in 0..48 {
            if self.keys[i] < 0 {
                full = false;
                break;
            }
        }
        full
    }
    
    // Node48
    fn need_shrink(&self) -> bool {
        let mut count = 0i32;
        for i in 0..16 {
            if self.keys[i] >= 0 {
                count += 1;
            }
        }
        if count < 15 {
            return true;
        }
        false
    }
    
    // Node48
    fn set_value(&mut self, val: String) {
        self.values.push(val);
    }
    
    // Node48
    fn get_values(&self) -> Option<Vec<String>> {
        let mut r: Vec<String> = Vec::new();
        r.clone_from(&self.values);
        Some(r)
    }
    
    // Node48
    fn del_values(&mut self) {
        self.values.clear();
    }
    
    // Node48
    fn set_parent(&mut self, pare: Option<Rc<RefCell<dyn ArtNode>>>) {
        self.parent = pare;
    }
    
    // Node48
    fn get_parent(&self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        match self.parent {
            Some(ref rc) => return Some(rc.clone()),
            None => return None,
        }
    }
}

impl ArtNode for Node256 {
    
    // Node256
    fn set_child(&mut self, key: u8, ch: Option<Rc<RefCell<dyn ArtNode>>>) -> bool {
        match self.children[key as usize] {
            Some(ref rc) => rc.borrow_mut().set_parent(None),
            None => {},
        }
        self.children[key as usize] = ch;
        true
    }

    // Node256
    fn get_child(&mut self, key: u8) -> Option<Rc<RefCell<dyn ArtNode>>> {
        match self.children[key as usize] {
            Some(ref rc) => return Some(rc.clone()),
            None => return None,
        }
    }

    // Node256
    fn del_child(&mut self, key: u8) {
        match self.children[key as usize] {
            Some(ref rc) => rc.borrow_mut().set_parent(None),
            None => {},
        }
        self.children[key as usize] = None;
    }
    
    // Node256
    fn empty(&self) -> bool {
        for i in 0..256 {
            match self.children[i] {
                Some(ref _rc) => return false,
                None => {},
            }
        }
        true 
    }
    
    // Node256
    fn get_children(&mut self) -> Vec<Option<Rc<RefCell<dyn ArtNode>>>> {
        let mut v: Vec<Option<Rc<RefCell<dyn ArtNode>>>> = Vec::new();
        v.clone_from(&self.children);
        v
    }

    // Node256
    fn shrink(&mut self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        let mut node48 = Node48::new(self.edge);
        
        // copy values to new node
        node48.values.clone_from(&mut self.values);
        
        // copy children to new node
        for i in 0..256 {
            match self.children[i] {
                Some(ref rc) => {
                    let key = rc.borrow().get_edge();
                    node48.children.push(Some(rc.clone()));
                    node48.keys[key as usize] = (node48.children.len()-1) as i8;
                },
                None => {},
            }
        }
        
        // set parent for new node
        match self.parent {
            Some(ref rc) => node48.parent = Some(rc.clone()),
            None => {
                println!("Parent node can't be None");
                return None;
            }
        }
        let rc = Rc::new(RefCell::new(node48));
        
        // set children's parent to new node
        let chs = rc.borrow_mut().get_children();
        for cn in chs.iter() {
            match *cn {
                Some(ref crc) => crc.borrow_mut().set_parent(Some(rc.clone())),
                None => {},
            }
        }
        Some(rc.clone())
    }

    // Node256
    fn get_edge(&self) -> u8 {
        self.edge
    }
    
    // Node256
    fn need_expand(&self) -> bool {
        false
    }
    
    // Node256
    fn need_shrink(&self) -> bool {
        let mut count = 0i32;
        for i in 0..256 {
            if self.children[i].is_some() {
                count += 1;
            }
        }
        if count < 46 {
            return true;
        }
        false
    }
    
    // Node256
    fn set_value(&mut self, val: String) {
        self.values.push(val);
    }
    
    // Node256
    fn get_values(&self) -> Option<Vec<String>> {
        let mut r: Vec<String> = Vec::new();
        r.clone_from(&self.values);
        Some(r)
    }
    
    // Node256
    fn del_values(&mut self) {
        self.values.clear();
    }
    
    // Node256
    fn set_parent(&mut self, pare: Option<Rc<RefCell<dyn ArtNode>>>) {
        self.parent = pare;
    }
    // Node256
    fn get_parent(&self) -> Option<Rc<RefCell<dyn ArtNode>>> {
        match self.parent {
            Some(ref rc) => return Some(rc.clone()),
            None => return None,
        }
    }
}

pub struct Sentinel {
    child: Option<Rc<RefCell<dyn ArtNode>>>,
}

impl Sentinel {
    pub fn new() -> Sentinel {
        Sentinel {
            child: None,
        }
    }

}

impl ArtNode for Sentinel {
    fn set_child(&mut self, _key: u8, ch: Option<Rc<RefCell<dyn ArtNode>>>) -> bool {
        match self.child {
            Some(ref rc) => rc.borrow_mut().set_parent(None),
            None => {},
        }
        self.child = ch;
        true
    }

    fn get_child(&mut self, _key: u8) -> Option<Rc<RefCell<dyn ArtNode>>> {
        match self.child {
            Some(ref rc) => return Some(rc.clone()),
            None => return None,
        }
    }

    fn del_child(&mut self, _key: u8) {
        match self.child {
            Some(ref rc) => rc.borrow_mut().set_parent(None),
            None => {},
        }
        self.child = None;
    }
}


