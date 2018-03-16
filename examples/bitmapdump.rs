// Copyright © 2015-2018, Peter Atashian
extern crate nx;
extern crate image;

use image::{ImageBuffer, Rgba};
use nx::node::{GenericNode};
use std::fs::{create_dir};
use std::path::{Path};

fn recurse<'a>(node: nx::Node<'a>, name: &str) {
    if let Some(bitmap) = node.bitmap() {
        let mut buf = vec![0; bitmap.len() as usize];
        bitmap.data(&mut buf);
        for chunk in buf.chunks_mut(4) {
            let t = chunk[0];
            chunk[0] = chunk[2];
            chunk[2] = t;
        }
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
            bitmap.width() as u32, bitmap.height() as u32, buf).unwrap();
        img.save(format!("bitmap/{}.png", name)).unwrap();
    }
    for child in node.iter() {
        recurse(child, &format!("{}.{}", name, child.name()))
    }
}

fn main() {
    let _ = create_dir("bitmap");
    let file = unsafe { nx::File::open(&Path::new("Data.nx")).unwrap() };
    recurse(file.root(), "Data");
}
