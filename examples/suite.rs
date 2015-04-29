// Copyright © 2014, Peter Atashian
#![feature(std_misc)]

extern crate nx;

use nx::GenericNode;
use std::path::Path;
use std::time::duration::Duration;

fn benchmark_suite() {
    fn load() -> nx::File {
        nx::File::open(&Path::new("Data.nx")).unwrap()
    }
    fn recurse(node: nx::Node) -> u32 {
        node.iter().fold(1, |a, b| a + recurse(b))
    }
    fn str_recurse(node: nx::Node) -> u32 {
        node.iter().fold(1, |a, b| {
            assert!(node.get(b.name()) == Some(b));
            a + str_recurse(b)
        })
    }
    fn test<F>(name: &str, count: u32, func: F) where F: Fn() -> u32 {
        let mut answer = 0;
        let mut vec = (0..count).map(|_| {
            Duration::span(|| answer = func()).num_microseconds().unwrap()
        }).collect::<Vec<_>>();
        vec.sort();
        let high = vec[vec.len() * 3 / 4];
        let slice = &vec[vec.len() * 1 / 4..vec.len() * 3 / 4];
        let mid = slice.iter().fold(0, |a, &b| a + b) / slice.len() as i64;
        let low = vec[0];
        println!("{}\t{}\t{}\t{}\t{}", name, high, mid, low, answer);
    }
    let file = nx::File::open(&Path::new("Data.nx")).unwrap();
    let node = file.root();
    println!("Name\t75%t\tM50%\tBest\tChecksum");
    test("Ld", 0x1000, || load().node_count() as u32);
    test("Re", 0x20, || recurse(node));
    test("LR", 0x20, || recurse(load().root()));
    test("SA", 0x20, || str_recurse(node));
}

fn main() {
    benchmark_suite()
}
