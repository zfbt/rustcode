mod trie;

fn main() {
    let mut t = trie::Trie::new();
    println!("Size of trie: {}", t.get_size());


    t.insert(String::from("hello"), String::from("trie"));
    t.insert(String::from("whyisrust"), String::from("sohard"));
    t.insert(String::from("hello"), String::from("sohard"));
    t.insert(String::from("k"), String::from("v"));

    match *t.get_root() {
        Some(ref node) => println!("root node: {}", node),
        None => {},
    }

    println!("The trie size: {}, contain keys:\n{}", t.get_size(), t);


    let res = t.search(&String::from("k"));
    match res {
        Some(ref vals) => {
            println!("Found values: ");
            for ele in vals {
                println!("{}", ele);
            }
        },
        None => println!("Key not exists"),
    }

}

