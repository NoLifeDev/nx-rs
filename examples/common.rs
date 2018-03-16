// Copyright © 2015-2018, Peter Atashian
extern crate nx;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::path::Path;

fn common_names<'a>(file: &'a nx::File) -> Vec<(&'a str, u32)> {
    let mut names = HashMap::new();
    fn recurse<'a>(names: &mut HashMap<&'a str, u32>, node: nx::Node<'a>) {
        match names.entry(node.name()) {
            Occupied(mut x) => *x.get_mut() += 1,
            Vacant(x) => drop(x.insert(1)),
        }
        for child in node.iter() { recurse(names, child) }
    }
    recurse(&mut names, file.root());
    let mut stuff: Vec<_> = names.iter().map(|(&key, &value)| (key, value)).collect();
    stuff.sort_by(|&(_, a), &(_, b)| a.cmp(&b));
    stuff
}

fn main() {
    let file = unsafe { nx::File::open(&Path::new(r"Data.nx")).unwrap() };
    let results = common_names(&file);
    for &(name, count) in results.iter() {
        println!("{}: {}", count, name);
    }
}
