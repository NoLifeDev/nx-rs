// Copyright © 2015-2018, Peter Atashian

/// An NX file header.
#[repr(packed)]
#[allow(dead_code)]
pub struct Header {
    pub magic: u32,
    pub nodecount: u32,
    pub nodeoffset: u64,
    pub stringcount: u32,
    pub stringoffset: u64,
    pub bitmapcount: u32,
    pub bitmapoffset: u64,
    pub audiocount: u32,
    pub audiooffset: u64,
}

/// The data contained by an NX node.
#[repr(packed)]
pub struct Node {
    pub name: u32,
    pub children: u32,
    pub count: u16,
    pub dtype: u16,
    pub data: u64,
}

/// The types of NX nodes.
#[repr(u16)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Type {
    /// A node containing no data.
    Empty = 0,
    /// A node containing integer data.
    Integer = 1,
    /// A node containing floating-point data.
    Float = 2,
    /// A node containing string data.
    String = 3,
    /// A node containing vector (or point) data.
    Vector = 4,
    /// A node containing bitmap data.
    Bitmap = 5,
    /// A node containing audio data.
    Audio = 6,
}

#[repr(packed)]
pub struct Integer {
    pub value: i64,
}

#[repr(packed)]
pub struct Float {
    pub value: f64,
}

#[repr(packed)]
pub struct String {
    pub index: u32,
    pub _padding: u32,
}

#[repr(packed)]
pub struct Vector {
    pub x: i32,
    pub y: i32,
}

#[repr(packed)]
#[allow(dead_code)]
pub struct Bitmap {
    pub index: u32,
    pub width: u16,
    pub height: u16,
}

#[repr(packed)]
#[allow(dead_code)]
pub struct Audio {
    pub index: u32,
    pub length: u32,
}
