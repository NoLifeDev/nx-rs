
use native::io::file::open;
use rustrt::rtio::{Open, Read, RtioFileStream};
use std::fmt;
use std::mem::transmute;
use std::os::{MapReadable, MapFd, MemoryMap};
use std::slice::raw;

macro_rules! try(
    ($e:expr, $f:expr) => (match $e { Ok(e) => e, Err(_) => return Err($f) })
)

pub struct File {
    map: MemoryMap,
    data: *const u8,
    header: *const Header,
    nodetable: *const NodeData,
    stringtable: *const u64,
}
impl File {
    pub fn open(path: &Path) -> Result<File, &'static str> {
        let mut file = try!(open(&path.to_c_str(), Open, Read), "Failed to open file");
        let stat = try!(file.fstat(), "Failed to get file size");
        let map = try!(MemoryMap::new(stat.size as uint, [MapReadable, MapFd(file.fd())]),
                       "Failed to map file");
        let data = map.data as *const u8;
        let header: *const Header = unsafe{transmute(data)};
        if unsafe{(*header).magic} != 0x34474B50 {
            return Err("Not a valid NX PKG4 file");
        }
        let nodetable: *const NodeData = unsafe{transmute(data.offset((*header).nodeoffset as int))};
        let stringtable: *const u64 = unsafe{transmute(data.offset((*header).stringoffset as int))};
        Ok(File{map: map, data: data, header: header, nodetable: nodetable,
                stringtable: stringtable})
    }
    pub fn header(&self) -> &Header {
        unsafe{transmute(self.header)}
    }
    pub fn root<'a>(&'a self) -> Node<'a> {
        Node{data: unsafe{&*self.nodetable}, file: self}
    }
    fn get_str<'a>(&'a self, index: u32) -> &'a str {
        let off = unsafe{*self.stringtable.offset(index as int)};
        let ptr = unsafe{self.data.offset(off as int)};
        let size: *const u16 = unsafe{transmute(ptr)};
        unsafe{raw::buf_as_slice(ptr.offset(2), (*size) as uint, |buf| {
            let bytes: &'a [u8] = transmute(buf);
            transmute(bytes)
        })}
    }
}
#[packed]
struct Header {
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
    pub fn iter(&self) -> NodeIterator<'a> {
        let data = unsafe{self.file.nodetable.offset(self.data.children as int)};
        NodeIterator{data: data, count: self.data.count, file: self.file}
    }
    pub fn name(&self) -> &'a str { self.file.get_str(self.data.name) }
    pub fn empty(&self) -> bool { self.data.count == 0 }
    pub fn get(&self, name: &'a str) -> Option<Node<'a>> {
        let mut data = unsafe{self.file.nodetable.offset(self.data.children as int)};
        let mut count = self.data.count as int;
        while count > 0 {
            let half = count / 2;
            let temp = unsafe{data.offset(half)};
            let other = self.file.get_str(unsafe{(*temp).name});
            match other.cmp(&name) {
                Less => { data = unsafe{temp.offset(1)}; count -= half + 1; },
                Equal => return Some(Node{data: unsafe{&*temp}, file: self.file}),
                Greater => count = half,
            }
        }
        None
    }
}
impl <'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Node) -> bool { self.data as *const _ == other.data as *const _ }
}
impl <'a> Eq for Node<'a> {}
impl <'a> fmt::Show for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
struct NodeIterator<'a> {
    data: *const NodeData,
    count: u16,
    file: &'a File,
}
impl <'a> Iterator<Node<'a>> for NodeIterator<'a> {
    fn next(&mut self) -> Option<Node<'a>> {
        match self.count {
            0 => None,
            _ => {
                self.count -= 1;
                let node = Node{data: unsafe{&*self.data}, file: self.file};
                self.data = unsafe{self.data.offset(1)};
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
