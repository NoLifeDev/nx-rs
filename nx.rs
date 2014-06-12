use std::fmt;

#[cfg(windows)]
mod mmap {
    use libc::{CreateFileW, CreateFileMappingW, MapViewOfFile, CloseHandle, UnmapViewOfFile,
               GENERIC_READ, FILE_SHARE_READ, OPEN_EXISTING, INVALID_HANDLE_VALUE, PAGE_READONLY,
               FILE_MAP_READ, HANDLE, TRUE};
    use libc::consts::os::extra::{FILE_FLAG_RANDOM_ACCESS};
    use libc::types::os::arch::extra::{LPCVOID};
    use std::ptr;
    struct Handle {
        hand: HANDLE,
    }
    impl Drop for Handle {
        fn drop(&mut self) {
            assert_eq!(unsafe { CloseHandle(self.hand) }, TRUE);
        }
    }
    struct MapView {
        data: LPCVOID,
    }
    impl Drop for MapView {
        fn drop(&mut self) {
            assert_eq!(unsafe { UnmapViewOfFile(self.data) }, TRUE);
        }
    }
    pub struct MapFile {
        view: MapView,
        map: Handle,
        file: Handle,
    }
    impl MapFile {
        pub fn data(&self) -> LPCVOID { self.view.data }
    }
    fn create_file(path: &Path) -> Result<Handle, &'static str> {
        let name = path.as_str().unwrap().to_utf16().append([0]);
        let file = unsafe {
            CreateFileW(name.as_ptr(), GENERIC_READ, FILE_SHARE_READ, ptr::mut_null(),
                        OPEN_EXISTING, FILE_FLAG_RANDOM_ACCESS, ptr::mut_null())
        };
        if file.to_uint() == INVALID_HANDLE_VALUE as uint { return Err("Failed to open file"); }
        Ok(Handle{hand: file})
    }
    fn create_mapping(file: &Handle) -> Result<Handle, &'static str> {
        let map = unsafe {
            CreateFileMappingW(file.hand, ptr::mut_null(), PAGE_READONLY, 0, 0, ptr::null())
        };
        if map.is_null() { return Err("Failed to create file mapping"); }
        Ok(Handle{hand: map})
    }
    fn map_view(map: &Handle) -> Result<MapView, &'static str> {
        let view = unsafe { MapViewOfFile(map.hand, FILE_MAP_READ, 0, 0, 0) };
        if view.is_null() { return Err("Failed to map view of file"); }
        Ok(MapView{data: view as *_})
    }
    pub fn open(path: &Path) -> Result<MapFile, &'static str> {
        let file = try!(create_file(path));
        let map = try!(create_mapping(&file));
        let view = try!(map_view(&map));
        Ok(MapFile{file: file, map: map, view: view})
    }
}
#[cfg(unix)]
mod mmap {
    use libc::{close, mmap, munmap, O_RDONLY, PROT_READ, c_int, c_void};
    use libc::consts::os::posix88::{MAP_SHARED};
    use posix_open = libc::open;
    use std::ptr;
    use std::io::fs::stat;
    struct Handle {
        hand: c_int,
    }
    impl Drop for Handle {
        fn drop(&mut self) {
            assert_eq!(unsafe { close(self.hand) }, 0);
        }
    }
    struct Mapping {
        data: *c_void,
        size: u64,
    }
    impl Drop for Mapping {
        fn drop(&mut self) {
            assert_eq!(unsafe { munmap(self.data, self.size) }, 0);
        }
    }
    pub struct MapFile {
        map: Mapping,
        file: Handle,
    }
    impl MapFile {
        pub fn data(&self) -> * c_void { self.map.data }
    }
    fn open_file(path: &Path) -> Result<Handle, &'static str> {
        let name = path.to_c_str();
        let handle = unsafe {
             posix_open(name.unwrap(), O_RDONLY, 0)
        };
        if handle == -1 { return Err("Failed to open file") }
        Ok(Handle{hand: handle})
    }
    fn file_size(path: &Path) -> Result<u64, &'static str> {
        match path.stat() {
            Ok(stat) => Ok(stat.size),
            Err(_) => Err("Failed to get file size")
        }
    }
    fn map_file(file: &Handle, size: u64) -> Result<Mapping, &'static str> {
        let map = unsafe {
            mmap(ptr::null(), size as u64, PROT_READ, MAP_SHARED, file.hand, 0)
        };
        if map.to_uint() == -1 { return Err("Failed to map file"); }
        Ok(Mapping{data: map as *_, size: size})
    }
    pub fn open(path: &Path) -> Result<MapFile, &'static str> {
        let file = try!(open_file(path));
        let size = try!(file_size(path));
        let map = try!(map_file(&file, size));
        Ok(MapFile{file: file, map: map})
    }
}
pub struct File {
    map: mmap::MapFile,
    data: *u8,
    header: *Header,
    nodetable: *NodeData,
    stringtable: *u64,
}
impl File {
    pub fn open(path: &Path) -> Result<File, &'static str> {
        use std::mem::transmute;
        let map = try!(mmap::open(path));
        let data = map.data();
        let header: *Header = unsafe{ transmute(data) };
        if unsafe { (*header).magic } != 0x34474B50 {
            return Err("Not a valid NX PKG4 file");
        }
        unsafe {
            Ok(File{map: map, data: transmute(data), header: header,
                    nodetable: transmute(data.offset((*header).nodeoffset as int)),
                    stringtable: transmute(data.offset((*header).stringoffset as int))})
        }
    }
    pub fn header(&self) -> &Header {
        use std::mem::transmute;
        unsafe { transmute(self.header) }
    }
    pub fn root<'a>(&'a self) -> Node<'a> {
        unsafe { Node{data: &*self.nodetable, file: self,} }
    }
    fn get_str<'a>(&'a self, index: u32) -> &'a str {
        use std::slice::raw;
        use std::mem::transmute;
        unsafe {
            let off = *self.stringtable.offset(index as int);
            let ptr = self.data.offset(off as int);
            let size: *u16 = transmute(ptr);
            raw::buf_as_slice(ptr.offset(2), (*size) as uint, |buf| {
                let bytes: &'a [u8] = transmute(buf);
                transmute(bytes)
            })
        }
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
        unsafe {
            let data = self.file.nodetable.offset(self.data.children as int);
            NodeIterator{data: data, count: self.data.count, file: self.file,}
        }
    }
    pub fn name(&self) -> &'a str { self.file.get_str(self.data.name) }
    pub fn empty(&self) -> bool { self.data.count == 0 }
    pub fn get(&self, name: &'a str) -> Option<Node<'a>> {
        if self.empty() { return None; }
        unsafe {
            let mut data = self.file.nodetable.offset(self.data.children as int);
            let mut count = self.data.count;
            while count > 0 {
                let half = count / 2;
                let temp = data.offset(half as int);
                let other = self.file.get_str((*temp).name);
                match other.cmp(&name) {
                    Less => { data = temp.offset(1); count -= half + 1; },
                    Equal => return Some(Node{data: &*temp, file: self.file}),
                    Greater => count = half,
                }
            }
            None
        }
    }
}
impl <'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Node) -> bool { self.data as *_ == other.data as *_ }
}
impl <'a> fmt::Show for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
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
                self.count -= 1;
                let node = Node{data: unsafe { &*self.data }, file: self.file};
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
