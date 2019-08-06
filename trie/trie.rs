use std::fmt;
use std::u8;

const TRIE_NODE_SPAN: usize = 256;

pub struct TrieNode {
    edge: u8,
    child: Vec<Option<Box<TrieNode>>>,
    values: Vec<String>,
}

impl TrieNode {

    fn new(edge: u8) -> TrieNode {
        let mut n = TrieNode {
            edge: edge,
            child: Vec::new(),
            values: Vec::new(),
        };
        n.child.resize_with(TRIE_NODE_SPAN, || { None });
        n
    }

}

pub struct Trie {
    root: Option<Box<TrieNode>>,
    size: u32,
    // mutex: Mutex<u8>, 
}

impl Trie {

    pub fn new() -> Box<Trie> {
        let trie = Trie {
            root: Some(Box::new(TrieNode::new(0))),
            size: 0,
            // mutex: Mutex::new(0),
        };
        return Box::new(trie);
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn get_root(&self) -> &Option<Box<TrieNode>> {
        &self.root
    }

    pub fn insert(&mut self, key: String, val: String) -> &Box<TrieNode> {
        let key_bytes = key.as_bytes();

        // assert(key_bytes.len() > 0)
        let mut curr_node = &mut self.root;

        for c in key_bytes.iter() {
            let idx = *c as usize;
            if (*curr_node).is_none() {
                println!("Internal error: current node can't be None");
                break;
            }
            let chs = (*curr_node).as_mut().map(|ptr| &mut (*ptr).child).unwrap();
            let node = &mut chs[idx];

            match *node {
                Some(_) => {
                    curr_node = node;
                },
                None => {
                    (*node).replace(Box::new(TrieNode::new(*c)));
                    curr_node = node; 
                    self.size += 1
                },
            }
        }

        (*curr_node).as_mut().map(|ptr| {
            (*ptr).values.push(val);
            ptr
        }).unwrap()
    }

    fn dfs(&self, s: &mut String) {
        let mut t = String::new();
        let mut node_st: Vec<&Option<Box<TrieNode>>> = Vec::new();
        let mut idx_st: Vec<u8> = Vec::new();
        
        if self.root.is_none() {
            println!("Internal error: root node can't be None");
            return;
        }

        let mut curr_node = &self.root;
        let mut curr_idx: u8 = 0;

        loop {
            match *curr_node {
                Some(ref ptr) => {
                    t.push(ptr.edge as char);
                    if !ptr.values.is_empty() {
                        s.push_str(t.as_str());
                        s.push('\n');
                    }
                    node_st.push(curr_node);
                    idx_st.push(curr_idx);
                    curr_node = &(*ptr).child[0];
                    curr_idx = 0;
                }
                None => {
                    if curr_idx == u8::MAX {
                        node_st.pop();
                        t.pop();
                        curr_idx = idx_st.pop().unwrap();
                        if node_st.is_empty() {
                            break;
                        }
                    }
                    let len = node_st.len();
                    let chs = (*node_st[len-1]).as_ref().map(|ptr| &(*ptr).child).unwrap();
                    let next_idx: usize = curr_idx as usize;
                    curr_node = &chs[next_idx + 1];
                    curr_idx = curr_idx + 1;
                }
            }
        }
    }

    pub fn search(&self, key: &String) -> Option<Vec<String>> {
        let key_bytes = key.as_bytes();

        let mut curr_node = &self.root;

        for c in key_bytes.iter() {
            let idx = *c as usize;
            if (*curr_node).is_none() {
                println!("Internal error: current node can't be None");
                break;
            }
            let chs = (*curr_node).as_ref().map(|ptr| &(*ptr).child).unwrap();
            let node = &chs[idx];

            match *node {
                Some(_) => {
                    curr_node = node;
                },
                None => {
                    return None;
                },
            }
        }

        let mut r: Vec<String> = Vec::new();

        (*curr_node).as_ref().map(|ptr| {
            r.clone_from(&(*ptr).values);
        });
        Some(r)
    }
}

impl fmt::Display for TrieNode {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut dis = format!("edge: {}, direct child: ", self.edge);

        for ele in &self.child {
            match ele {
                Some(ref ptr) => {
                    dis.push(ptr.edge as char);
                    dis.push(' ');
                },
                None => {},
            }
        }
        write!(f, "{}", dis.as_str())
    }
}

impl fmt::Display for Trie {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new(); 
        self.dfs(&mut s);       

        write!(f, "{}", s.as_str())
    }

}





















