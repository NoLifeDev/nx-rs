
extern crate libc;
extern crate time;
use std::{ptr};
use std::mem::{transmute};

struct Handle;
struct SecurityAttributes;

extern "system" {
	fn CreateFileW(filename: *u16, access: u32, share: u32, security: *SecurityAttributes, creation: u32, flags: u32, template: *Handle) -> *Handle;
	fn CreateFileMappingW(file: *Handle, security: *SecurityAttributes, protect: u32, sizehigh: u32, sizelow: u32, name: *u16) -> *Handle;
	fn MapViewOfFile(map: *Handle, access: u32, offsethigh: u32, offsetlow: u32, size: uint) -> *u8;
	fn GetLastError() -> u32;
}

static GENERIC_READ: u32 = 0x80000000;
static FILE_SHARE_READ: u32 = 0x00000001;
static OPEN_EXISTING: u32 = 3;
static FILE_FLAG_RANDOM_ACCESS: u32 = 0x10000000;
static PAGE_READONLY: u32 = 0x02;
static SECTION_MAP_READ: u32 = 0x0004;
static FILE_MAP_READ: u32 = SECTION_MAP_READ;
static INVALID_HANDLE_VALUE: uint = -1;

struct File {
	data: *u8,
	header: *Header,
	nodetable: *NodeData,
	stringtable: *u64
}
impl File {
	fn open(name: &Path) -> Option<File> {
		unsafe {
			let sname = name.as_str().unwrap().to_utf16();
			let handle = CreateFileW(sname.as_ptr(), GENERIC_READ, FILE_SHARE_READ, ptr::null(), OPEN_EXISTING, FILE_FLAG_RANDOM_ACCESS, ptr::null());
			if handle.to_uint() == INVALID_HANDLE_VALUE {
				return None;
			}
			let map = CreateFileMappingW(handle, ptr::null(), PAGE_READONLY, 0, 0, ptr::null());
			if map.is_null() {
				return None;
			}
			let data = MapViewOfFile(map, FILE_MAP_READ, 0, 0, 0);
			if data.is_null() {
				return None;
			}
			let header: *Header = transmute(data);
			if (*header).magic != 0x34474B50 {
				return None;
			}
			let file = File {
				data: data,
				header: header,
				nodetable: transmute(data.offset((*header).nodeoffset.to_int().unwrap())),
				stringtable: transmute(data.offset((*header).stringoffset.to_int().unwrap()))
			};
			return Some(file);
		} 
	}
	fn get_header(&self) -> &Header {
		unsafe {
			transmute(self.header)
		}
	}
	fn root<'a>(&'a self) -> Node<'a> {
		unsafe {
			Node {data: &*self.nodetable, file: self}
		}
	}
	fn get_str<'a>(&'a self, index: u32) -> Option<&'a str> {
		unsafe {
			let ptr = self.data.offset((*self.stringtable.offset(index.to_int().unwrap())).to_int().unwrap());
			let size: *u16 = transmute(ptr);
			std::slice::raw::buf_as_slice(ptr.offset(2), (*size).to_uint().unwrap(), |buf: &[u8]| -> Option<&'a str> {
				let bytes: &'a [u8] = transmute(buf);
				std::str::from_utf8(bytes)
			})
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
	audiooffset: u64
}
#[packed]
struct String {
	length: u16
}
struct Node<'a> {
	data: &'a NodeData,
	file: &'a File
}
impl<'a> Node<'a> {
	fn iter(&self) -> NodeIterator<'a> {
		unsafe {
			NodeIterator {data: self.file.nodetable.offset(self.data.children as int), count: self.data.count, file: self.file}
		}
	}
	fn name(&self) -> Option<&'a str> {
		self.file.get_str(self.data.name)
	}
	fn empty(&self) -> bool {
		self.data.count == 0
	}
}
struct NodeIterator<'a> {
	data: *NodeData,
	count: u16,
	file: &'a File
}
impl<'a> Iterator<Node<'a>> for NodeIterator<'a> {
	fn next(&mut self) -> Option<Node<'a>> {
		match self.count {
			0 => None,
			_ => {
				self.count -= 1;
				let node = Node {data: unsafe {&*self.data}, file: self.file};
				self.data = unsafe {self.data.offset(1)};
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
	data: u64
}
#[packed]
struct NodeInteger {
	value: i64
}
#[packed]
struct NodeFloat {
	value: f64
}
#[packed]
struct NodeString {
	index: u32,
	unused: u32
}
#[packed]
struct NodeVector {
	x: i32,
	y: i32
}
#[packed]
struct NodeBitmap {
	index: u32,
	width:  u16,
	height: u16
}
#[packed]
struct NodeAudio {
	index: u32,
	length: u32
}

fn recurse(node: Node) -> int {
	node.iter().fold(1, |a, b| a + recurse(b))
}

fn test(name: &str, func: |Node| -> int, node: Node) {
	for i in range(0, 10) {
		let begin = time::precise_time_ns();
		let answer = recurse(node);
		let end = time::precise_time_ns();
		println!("{}\t{}\t{}", name, (end - begin) / 1000, answer);
	}
}

fn main() {
	unsafe { ::std::rt::stack::record_sp_limit(0); }
	let file = File::open(&Path::new("Data.nx")).unwrap();
	test("Re", recurse, file.root());
}

