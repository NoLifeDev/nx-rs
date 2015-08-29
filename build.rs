// Copyright © 2015, Peter Atashian
extern crate gcc;
fn main() {
    gcc::compile_library("liblz4.a", &["src/lz4.c"]);
}
