
use std::{ptr};
use std::mem::{transmute};
pub struct File {
    data: *u8,
    header: *Header,
    nodetable: *NodeData,
    stringtable: *u64,
}
impl File {
    #[cfg(windows)]
    pub fn open(path: &Path) -> Option<File> {
        use libc::funcs::extra::kernel32::{CreateFileW, CreateFileMappingW,
                                           MapViewOfFile};
        use libc::consts::os::extra::{GENERIC_READ, FILE_SHARE_READ,
                                      OPEN_EXISTING, FILE_FLAG_RANDOM_ACCESS,
                                      INVALID_HANDLE_VALUE, PAGE_READONLY,
                                      FILE_MAP_READ};
        unsafe {
            let mut name = path.as_str().unwrap().to_utf16();
            name.push(0);
            let handle =
                CreateFileW(name.as_ptr(), GENERIC_READ, FILE_SHARE_READ,
                            ptr::mut_null(), OPEN_EXISTING,
                            FILE_FLAG_RANDOM_ACCESS, ptr::mut_null());
            if handle == transmute(INVALID_HANDLE_VALUE) { return None; }
            let map =
                CreateFileMappingW(handle, ptr::mut_null(), PAGE_READONLY, 0,
                                   0, ptr::null());
            if map.is_null() { return None; }
            let data = MapViewOfFile(map, FILE_MAP_READ, 0, 0, 0);
            if data.is_null() { return None; }
            let header: *Header = transmute(data);
            if (*header).magic != 0x34474B50 { return None; }
            let file =
                File{data: transmute(data),
                     header: header,
                     nodetable:
                         transmute(data.offset((*header).nodeoffset as int)),
                     stringtable:
                         transmute(data.offset((*header).stringoffset as
                                                   int)),};
            Some(file)
        }
    }
    #[cfg(unix)]
    pub fn open(path: &Path) -> Option<File> {
        use libc::funcs::posix88::fcntl::open;
        use libc::funcs::posix88::stat_::fstat;
        use libc::funcs::posix88::mman::mmap;
        use libc::consts::os::posix88::{O_RDONLY, PROT_READ, MAP_SHARED};
        use libc::types::os::arch::posix01::stat;
        unsafe {
            let name = name.to_c_str();
            let handle = open(name.as_bytes().as_ptr(), O_RDONLY);
            if (handle == -1) { return None; }
            let finfo: stat;
            if (fstat(handle, &finfo) == -1) { return None; }
            let size = finfo.st_size;
            let data =
                mmap(ptr::null(), size, PROT_READ, MAP_SHARED, handle, 0);
            if (data == -1) { return None; }
            let header: *Header = transmute(data);
            if (*header).magic != 0x34474B50 { return None; }
            let file =
                File{data: transmute(data),
                     header: header,
                     nodetable:
                         transmute(data.offset((*header).nodeoffset as int)),
                     stringtable:
                         transmute(data.offset((*header).stringoffset as
                                                   int)),};
            Some(file)
        }
    }
    fn get_header(&self) -> &Header { unsafe { transmute(self.header) } }
    pub fn root<'a>(&'a self) -> Node<'a> {
        unsafe { Node{data: &*self.nodetable, file: self,} }
    }
    fn get_str<'a>(&'a self, index: u32) -> Option<&'a str> {
        use std::str;
        use std::slice::raw;
        unsafe {
            let off = *self.stringtable.offset(index as int);
            let ptr = self.data.offset(off as int);
            let size: *u16 = transmute(ptr);
            let func = |buf: &[u8]| -> Option<&'a str> {
                let bytes: &'a [u8] = transmute(buf); str::from_utf8(bytes) };
            raw::buf_as_slice(ptr.offset(2), (*size) as uint, func)
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
pub struct Node<'a> {
    data: &'a NodeData,
    file: &'a File,
}
impl <'a> Node<'a> {
    pub fn iter(&self) -> NodeIterator<'a> {
        unsafe {
            let data = self.file.nodetable.offset(self.data.children as int);
            NodeIterator{data: data, count: self.data.count, file: self.file,}
        }
    }
    pub fn name(&self) -> Option<&'a str> {
        self.file.get_str(self.data.name)
    }
    pub fn empty(&self) -> bool { self.data.count == 0 }
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
