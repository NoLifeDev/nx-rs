// Copyright © 2014, Peter Atashian
//! A high performance Rust library used to read [NX files](http://nxformat.github.io/) with 
//! minimal memory usage.
#![warn(missing_docs)]
#![unstable]

use std::error::Error as StdError;
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

/// An error occuring anywhere in the library.
#[deriving(Show)]
pub enum Error {
    /// An internal IoError.
    IoError(IoError),
    /// An internal MapError.
    MapError(MapError),
    /// A library error. 
    NxError(&'static str),
}
impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            &Error::IoError(ref e) => e.description(),
            &Error::MapError(ref e) => e.description(),
            &Error::NxError(s) => s,
        }
    }
    fn detail(&self) -> Option<String> {
        match self {
            &Error::IoError(ref e) => e.detail(),
            &Error::MapError(ref e) => e.detail(),
            &Error::NxError(_) => None,  
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            &Error::IoError(ref e) => e.cause(),
            &Error::MapError(ref e) => e.cause(),
            &Error::NxError(_) => None,  
        }
    }
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
/// The standard result type used throughout the library.
pub type Result<T> = StdResult<T, Error>;

/// A memory-mapped NX file.
pub struct File {
    #[allow(dead_code)]
    map: MemoryMap,
    data: *const u8,
    header: *const Header,
    nodetable: *const node::Data,
    stringtable: *const u64,
}

impl File {
    /// Opens an NX file via memory-mapping. This also checks the magic bytes in the header.
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
    /// Gets the file header.
    #[inline]
    pub fn header(&self) -> &Header {
        unsafe { transmute(self.header) }
    }
    /// Gets the root node of the file.
    #[inline]
    pub fn root<'a>(&'a self) -> Node<'a> {
        unsafe { Node::construct(&*self.nodetable, self) }
    }
    /// Gets the string at the specified index in the string table.
    #[inline]
    fn get_str<'a>(&'a self, index: u32) -> &'a str {
        let off = unsafe { *self.stringtable.offset(index as int) };
        let ptr = unsafe { self.data.offset(off as int) };
        let size: *const u16 = unsafe { transmute(ptr) };
        unsafe { transmute(from_raw_buf(&ptr.offset(2), (*size) as uint)) }
    }
}

/// An NX file header.
#[repr(packed)]
#[allow(dead_code, missing_copy_implementations)]
pub struct Header {
    magic: u32,
    /// The number of nodes in the NX file.
    pub nodecount: u32,
    nodeoffset: u64,
    stringcount: u32,
    stringoffset: u64,
    bitmapcount: u32,
    bitmapoffset: u64,
    audiocount: u32,
    audiooffset: u64,
}
