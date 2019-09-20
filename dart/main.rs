mod art;

use std::i32;
use art::skiplist::{SkipList, Value};

impl Value for i32 {
    fn value(&self) -> u16 {
        *self as u16
    }
}

fn main() {
    println!("Hello, world!");
    println!("max i32: {}", i32::MAX);

    let mut sl = SkipList::new(32);
    for i in 56..85 {
        sl.insert(i as i32);
    }
    println!("skip list length: {}", sl.len());
    sl.print(0);
    /* 
    sl.print(1);
    sl.print(2);
    sl.print(3);
    sl.print(4);
    sl.print(5);
    sl.print(6);
    sl.print(7);
    sl.print(8);
    sl.print(9);
    */

    let t = 84i32;
    let res = sl.get(t);
    match res {
        Some(v) => println!("fonnd: {}", v.value()),
        None => println!("nothing"),
    }
    
    println!("skip list length: {}", sl.len());
    sl.print(0);
}
