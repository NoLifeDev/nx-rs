extern crate libc;
extern crate time;
use std::{ptr};
use std::mem::{transmute};
#[cfg(target_os = "win32")]
mod win {
    pub struct Handle;
    pub struct SecurityAttributes;
    extern "system" {
        pub fn CreateFileW(filename: *u16, access: u32, share: u32,
                           security: *SecurityAttributes, creation: u32,
                           flags: u32, template: *Handle) -> *Handle;
        pub fn CreateFileMappingW(file: *Handle,
                                  security: *SecurityAttributes, protect: u32,
                                  sizehigh: u32, sizelow: u32, name: *u16) ->
         *Handle;
        pub fn MapViewOfFile(map: *Handle, access: u32, offsethigh: u32,
                             offsetlow: u32, size: uint) -> *u8;
        pub fn GetLastError() -> u32;
    }
    pub static GENERIC_READ: u32 = 0x80000000;
    pub static FILE_SHARE_READ: u32 = 0x00000001;
    pub static OPEN_EXISTING: u32 = 3;
    pub static FILE_FLAG_RANDOM_ACCESS: u32 = 0x10000000;
    pub static PAGE_READONLY: u32 = 0x02;
    pub static SECTION_MAP_READ: u32 = 0x0004;
    pub static FILE_MAP_READ: u32 = SECTION_MAP_READ;
    pub static INVALID_HANDLE_VALUE: uint = -1;
}
#[cfg(target_os = "linux")]
mod posix {
    //Hi
}
struct File {
    data: *u8,
    header: *Header,
    nodetable: *NodeData,
    stringtable: *u64,
}
impl File {
    #[cfg(target_os = "win32")]
    fn open(name: &Path) -> Option<File> {
        unsafe {
            let sname = name.as_str().unwrap().to_utf16();
            let handle =
                win::CreateFileW(sname.as_ptr(), win::GENERIC_READ,
                                 win::FILE_SHARE_READ, ptr::null(),
                                 win::OPEN_EXISTING,
                                 win::FILE_FLAG_RANDOM_ACCESS, ptr::null());
            if handle.to_uint() == win::INVALID_HANDLE_VALUE { return None; }
            let map =
                win::CreateFileMappingW(handle, ptr::null(),
                                        win::PAGE_READONLY, 0, 0,
                                        ptr::null());
            if map.is_null() { return None; }
            let data = win::MapViewOfFile(map, win::FILE_MAP_READ, 0, 0, 0);
            if data.is_null() { return None; }
            let header: *Header = transmute(data);
            if (*header).magic != 0x34474B50 { return None; }
            let file =
                File{data: data,
                     header: header,
                     nodetable:
                         transmute(data.offset((*header).nodeoffset as int)),
                     stringtable:
                         transmute(data.offset((*header).stringoffset as
                                                   int)),};
            Some(file)
        }
    }
    #[cfg(target_os = "linux")]
    fn open(name: &Path) -> Option<File> {
        unsafe {
            //Someone fill this in please
        }
    }
    fn get_header(&self) -> &Header { unsafe { transmute(self.header) } }
    fn root<'a>(&'a self) -> Node<'a> {
        unsafe { Node{data: &*self.nodetable, file: self,} }
    }
    fn get_str<'a>(&'a self, index: u32) -> Option<&'a str> {
        unsafe {
            let off = *self.stringtable.offset(index as int);
            let ptr = self.data.offset(off as int);
            let size: *u16 = transmute(ptr);
            let func = |buf: &[u8]| -> Option<&'a str> {
                let bytes: &'a [u8] = transmute(buf);
                std::str::from_utf8(bytes) };
            std::slice::raw::buf_as_slice(ptr.offset(2), (*size) as uint,
                                          func)
        }
    }
}
#[packed]
struct Header {
    magic: u32,
    nodecount: u32,
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
struct Node<'a> {
    data: &'a NodeData,
    file: &'a File,
}
impl <'a> Node<'a> {
    fn iter(&self) -> NodeIterator<'a> {
        unsafe {
            let data = self.file.nodetable.offset(self.data.children as int);
            NodeIterator{data: data, count: self.data.count, file: self.file,}
        }
    }
    fn name(&self) -> Option<&'a str> { self.file.get_str(self.data.name) }
    fn empty(&self) -> bool { self.data.count == 0 }
}
struct NodeIterator<'a> {
    data: *NodeData,
    count: u16,
    file: &'a File,
}
impl <'a> Iterator<Node<'a>> for NodeIterator<'a> {
    fn next(&mut self) -> Option<Node<'a>> {
        match self.count {
            0 => None,
            _ => {
                unsafe {
                    self.count -= 1;
                    let node = Node{data: &*self.data, file: self.file,};
                    self.data = self.data.offset(1);
                    Some(node)
                }
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
fn recurse(node: Node) -> int { node.iter().fold(1, |a, b| a + recurse(b)) }
fn test(name: &str, func: |Node| -> int, node: Node) {
    for i in range(0, 10) {
        let begin = time::precise_time_ns();
        let answer = recurse(node);
        let end = time::precise_time_ns();
        println!("{}\t{}\t{}" , name , ( end - begin ) / 1000 , answer);
    }
}
fn main() {
    unsafe { ::std::rt::stack::record_sp_limit(0); }
    let file = File::open(&Path::new("Data.nx")).unwrap();
    test("Re", recurse, file.root());
}
