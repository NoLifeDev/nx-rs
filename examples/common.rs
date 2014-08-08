
extern crate nx;

use std::collections::HashMap;

fn common_names<'a>(file: &'a nx::File) -> Vec<(&'a str, uint)> {
    let mut names: HashMap<&'a str, uint> = HashMap::new();
    fn recurse<'a>(names: &mut HashMap<&'a str, uint>, node: nx::Node<'a>) {
        node.name().map(|s|
            names.insert_or_update_with(s, 1, |_, value| *value += 1)
        );
        for child in node.iter() { recurse(names, child) }
    }
    recurse(&mut names, file.root());
    let mut stuff: Vec<(&'a str, uint)> = names.iter()
        .map(|(&key, &value)| (key, value)).collect();
    stuff.sort_by(|&(_, a), &(_, b)| a.cmp(&b).reverse());
    stuff
}

fn main() {
    let file = nx::File::open(&Path::new("data.nx")).unwrap();
    let results = common_names(&file);
    for &(name, count) in results.iter() {
        println!("{}: {}", count, name);
    }
}
