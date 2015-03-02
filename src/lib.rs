// Copyright © 2014, Peter Atashian
//! A high performance Rust library used to read [NX files](http://nxformat.github.io/) with
//! minimal memory usage.
#![feature(core, old_io, old_path, os, std_misc)]
#![warn(missing_docs)]
#![unstable]

use std::error::Error as StdError;
use std::error::FromError;
use std::fmt::{Display, Formatter};
use std::fmt::Error as FmtError;
use std::old_io::fs::File as FsFile;
use std::old_io::IoError;
use std::mem::transmute;
use std::os::{MapError, MemoryMap};
use std::os::MapOption::{self, MapFd, MapReadable};
use std::result::Result;
use std::slice::from_raw_parts;

pub use node::Node;
pub use node::GenericNode;
pub use node::Type;

mod node;

/// An error occuring anywhere in the library.
#[derive(Debug)]
pub enum Error {
    /// An internal IoError.
    Io(IoError),
    /// An internal MapError.
    Map(MapError),
    /// A library error.
    Nx(&'static str),
}
impl StdError for Error {
    fn description(&self) -> &str {
        "Failed to load NX file"
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            &Error::Io(ref e) => Some(e),
            &Error::Map(ref e) => Some(e),
            &Error::Nx(_) => None,
        }
    }
}
impl FromError<IoError> for Error {
    fn from_error(err: IoError) -> Error {
        Error::Io(err)
    }
}
impl FromError<MapError> for Error {
    fn from_error(err: MapError) -> Error {
        Error::Map(err)
    }
}
impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        match self.cause() {
            Some(cause) => write!(fmt, "{} ({})", self.description(), cause),
            None => write!(fmt, "{}", self.description()),
        }
    }
}

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
    pub fn open(path: &Path) -> Result<File, Error> {
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
        let map = try!(MemoryMap::new(stat.size as usize, &[MapReadable, get_fd(&file)]));
        let data = map.data() as *const u8;
        let header: *const Header = unsafe { transmute(data) };
        if unsafe { (*header).magic } != 0x34474B50 {
            return Err(Error::Nx("Magic value is invalid"));
        }
        let nodetable: *const node::Data = unsafe {
            transmute(data.offset((*header).nodeoffset as isize))
        };
        let stringtable: *const u64 = unsafe {
            transmute(data.offset((*header).stringoffset as isize))
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
        let off = unsafe { *self.stringtable.offset(index as isize) };
        let ptr = unsafe { self.data.offset(off as isize) };
        let size: *const u16 = unsafe { transmute(ptr) };
        unsafe { transmute(from_raw_parts(ptr.offset(2), (*size) as usize)) }
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
