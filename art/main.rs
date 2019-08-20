mod art;

use art::Art;


fn main() {
    let mut art = Art::new();

    /*
    art.insert(String::from("hello"), String::from("art"));
    art.insert(String::from("hello"), String::from("art"));

    search(&art, &String::from("hello"));
    println!("After insert two keys, Art node size: {}", art.get_size());

    art.delete(&String::from("hello"));
    search(&art, &String::from("hello"));
    println!("After delete keys, Art node size: {}", art.get_size());
    */

    // make node expand
    art.insert(String::from("hello"), String::from("hello"));
    art.insert(String::from("helmo"), String::from("helmo"));
    art.insert(String::from("helno"), String::from("helno"));
    art.insert(String::from("heloo"), String::from("heloo"));
    art.insert(String::from("helpo"), String::from("helpo"));
    art.insert(String::from("helqo"), String::from("helqo"));
    
    search(&art, &String::from("hello"));
    search(&art, &String::from("helmo"));
    search(&art, &String::from("helno"));
    search(&art, &String::from("heloo"));
    println!("After node expand, Art node size: {}", art.get_size());


    art.delete(&String::from("hello"));
    art.delete(&String::from("helmo"));
    art.delete(&String::from("helno"));
    art.delete(&String::from("heloo"));
    art.delete(&String::from("helpo"));
    search(&art, &String::from("helqo"));
    println!("After node shrink, Art node size: {}", art.get_size());
}

fn search(art: &Art, key: &String) {
    let res = art.search(key);
    match res {
        Some(ref vals) => {
            println!("Found values for key:{} ", key);
            for ele in vals {
                println!("{}", ele);
            }
        },
        None => println!("Key:{} not exists", key),
    }
}

