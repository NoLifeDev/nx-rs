
#![crate_type = "rlib"]

extern crate rustrt;
extern crate native;

use native::io::file::open;
use rustrt::rtio::{Open, Read, RtioFileStream};
use std::fmt;
use std::mem::transmute;
use std::os::{MapReadable, MapFd, MemoryMap};
use std::slice::raw;
use std::str::from_utf8;

pub struct File {
    map: MemoryMap,
    data: *const u8,
    header: *const Header,
    nodetable: *const NodeData,
    stringtable: *const u64,
}

impl File {
    #[inline]
    pub fn open(path: &Path) -> Result<File, &'static str> {
        let mut file = match open(&path.to_c_str(), Open, Read) {
            Ok(file) => file,
            Err(_) => return Err("Failed to open file"),
        };
        let stat = match file.fstat() {
            Ok(stat) => stat,
            Err(_) => return Err("Failed to get file size"),
        };
        let map = match MemoryMap::new(stat.size as uint, [MapReadable, MapFd(file.fd())]) {
            Ok(map) => map,
            Err(_) => return Err("Failed to map file"),
        };
        let data = map.data as *const u8;
        let header: *const Header = unsafe { transmute(data) };
        if unsafe { (*header).magic } != 0x34474B50 {
            return Err("Not a valid NX PKG4 file");
        }
        let nodetable: *const NodeData = unsafe {
            transmute(data.offset((*header).nodeoffset as int))
        };
        let stringtable: *const u64 = unsafe {
            transmute(data.offset((*header).stringoffset as int))
        };
        Ok(File {
            map: map,
            data: data,
            header: header,
            nodetable: nodetable,
            stringtable: stringtable,
        })
    }
    #[inline]
    pub fn header(&self) -> &Header {
        unsafe { transmute(self.header) }
    }
    #[inline]
    pub fn root<'a>(&'a self) -> Node<'a> {
        Node {
            data: unsafe { &*self.nodetable },
            file: self,
        }
    }
    #[inline]
    fn get_str<'a>(&'a self, index: u32) -> &'a [u8] {
        let off = unsafe { *self.stringtable.offset(index as int) };
        let ptr = unsafe { self.data.offset(off as int) };
        let size: *const u16 = unsafe { transmute(ptr) };
        unsafe { raw::buf_as_slice(ptr.offset(2), (*size) as uint, |buf| {
            transmute(buf)
        }) }
    }
}

#[packed]
pub struct Header {
    magic: u32,
    pub nodecount: u32,
    nodeoffset: u64,
    stringcount: u32,
    stringoffset: u64,
    bitmapcount: u32,
    bitmapoffset: u64,
    audiocount: u32,
    audiooffset: u64,
}

#[packed]
struct String {
    length: u16,
}

pub struct Node<'a> {
    data: &'a NodeData,
    file: &'a File,
}

impl <'a> Node<'a> {
    #[inline]
    pub fn iter(&self) -> Nodes<'a> {
        let data = unsafe {
            self.file.nodetable.offset(self.data.children as int)
        };
        Nodes {
            data: data,
            count: self.data.count,
            file: self.file
        }
    }
    #[inline]
    pub fn name(&self) -> Option<&'a str> { from_utf8(self.file.get_str(self.data.name)) }
    #[inline]
    pub fn name_raw(&self) -> &'a [u8] { self.file.get_str(self.data.name) }
    #[inline]
    pub fn empty(&self) -> bool { self.data.count == 0 }
    #[inline]
    pub fn get(&self, name: &str) -> Option<Node<'a>> {
        self.get_raw(name.as_bytes())
    }
    #[inline]
    pub fn get_raw(&self, name: &[u8]) -> Option<Node<'a>> {
        let mut data = unsafe {
            self.file.nodetable.offset(self.data.children as int)
        };
        let mut count = self.data.count as int;
        while count > 0 {
            let half = count / 2;
            let temp = unsafe { data.offset(half) };
            let other = self.file.get_str(unsafe { (*temp).name });
            match other.cmp(&name) {
                Less => {
                    data = unsafe { temp.offset(1) };
                    count -= half + 1;
                },
                Equal => return Some(Node {
                    data: unsafe { &*temp },
                    file: self.file
                }),
                Greater => count = half,
            }
        }
        None
    }
}

impl <'a> PartialEq for Node<'a> {
    #[inline]
    fn eq(&self, other: &Node) -> bool {
        self.data as *const NodeData == other.data as *const NodeData
    }
}

impl <'a> Eq for Node<'a> {}

impl <'a> fmt::Show for Node<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

pub struct Nodes<'a> {
    data: *const NodeData,
    count: u16,
    file: &'a File,
}

impl <'a> Iterator<Node<'a>> for Nodes<'a> {
    #[inline]
    fn next(&mut self) -> Option<Node<'a>> {
        match self.count {
            0 => None,
            _ => {
                self.count -= 1;
                let node = Node {
                    data: unsafe { &*self.data },
                    file: self.file
                };
                self.data = unsafe { self.data.offset(1) };
                Some(node)
            }
        }
    }
}

#[packed]
struct NodeData {
    name: u32,
    children: u32,
    count: u16,
    dtype: u16,
    data: u64,
}

#[packed]
struct NodeInteger {
    value: i64,
}

#[packed]
struct NodeFloat {
    value: f64,
}

#[packed]
struct NodeString {
    index: u32,
    unused: u32,
}

#[packed]
struct NodeVector {
    x: i32,
    y: i32,
}

#[packed]
struct NodeBitmap {
    index: u32,
    width: u16,
    height: u16,
}

#[packed]
struct NodeAudio {
    index: u32,
    length: u32,
}
