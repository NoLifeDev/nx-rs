// Copyright © 2015-2018, Peter Atashian
extern crate cc;
fn main() {
    cc::Build::new().file("src/lz4.c").compile("lz4");
}
