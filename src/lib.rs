// Copyright © 2014, Peter Atashian

use std::error::FromError;
use std::io::fs::File as FsFile;
use std::io::IoError;
use std::mem::transmute;
use std::os::{MapError, MemoryMap};
use std::os::MapOption::{mod, MapFd, MapReadable};
use std::result::Result as StdResult;
use std::slice::from_raw_buf;

pub use node::Node;
pub use node::GenericNode;
pub use node::Type;

mod node;

#[deriving(Show)]
pub enum Error {
    IoError(IoError),
    MapError(MapError),
    NxError(&'static str),
}
impl FromError<IoError> for Error {
    fn from_error(err: IoError) -> Error {
        Error::IoError(err)
    }
}
impl FromError<MapError> for Error {
    fn from_error(err: MapError) -> Error {
        Error::MapError(err)
    }
}
pub type Result<T> = StdResult<T, Error>;

pub struct File {
    #[allow(dead_code)]
    map: MemoryMap,
    data: *const u8,
    header: *const Header,
    nodetable: *const node::Data,
    stringtable: *const u64,
}

impl File {
    pub fn open(path: &Path) -> Result<File> {
        let file = try!(FsFile::open(path));
        let stat = try!(file.stat());
        #[cfg(not(windows))]
        fn get_fd(file: &FsFile) -> MapOption {
            use std::os::unix::AsRawFd;
            MapFd(file.as_raw_fd())
        }
        #[cfg(windows)]
        fn get_fd(file: &FsFile) -> MapOption {
            use std::os::windows::AsRawHandle;
            MapFd(file.as_raw_handle())
        }
        let map = try!(MemoryMap::new(stat.size as uint, &[MapReadable, get_fd(&file)]));
        let data = map.data() as *const u8;
        let header: *const Header = unsafe { transmute(data) };
        if unsafe { (*header).magic } != 0x34474B50 {
            return Err(Error::NxError("Not a valid NX PKG4 file"));
        }
        let nodetable: *const node::Data = unsafe {
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
        unsafe { Node::construct(&*self.nodetable, self) }
    }
    #[inline]
    fn get_str<'a>(&'a self, index: u32) -> &'a str {
        let off = unsafe { *self.stringtable.offset(index as int) };
        let ptr = unsafe { self.data.offset(off as int) };
        let size: *const u16 = unsafe { transmute(ptr) };
        unsafe { transmute(from_raw_buf(&ptr.offset(2), (*size) as uint)) }
    }
}

#[repr(packed)]
#[allow(dead_code, missing_copy_implementations)]
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
