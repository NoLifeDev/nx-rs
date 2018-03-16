// Copyright © 2015-2018, Peter Atashian
extern crate nx;

use nx::GenericNode;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::Path;

fn main() {
    let file = unsafe { nx::File::open(&Path::new("Sound.nx")).unwrap() };
    for node in file.root().iter() {
        let group = node.name();
        if &group[..3] != "Bgm" { continue }
        let p = Path::new("Bgm").join(group);
        let _ = create_dir_all(&p);
        for song in node.iter() {
            if let Some(audio) = song.audio() {
                let mut pp = p.join(song.name());
                pp.set_extension("mp3");
                let mut file = File::create(pp).unwrap();
                file.write_all(audio.data()).unwrap();
            }
        }
    }
}
