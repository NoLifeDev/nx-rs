// Copyright © 2014, Peter Atashian

use std::cmp::Ordering::{Equal, Greater, Less};
use std::mem::transmute;
use std::num::FromPrimitive;

/// The basic functionality for all nodes. 
pub trait GenericNode<'a> {
    /// Gets the child node of the specified name.
    fn get(&self, name: &str) -> Option<Node<'a>>;
    /// Gets the type of this node.
    fn dtype(&self) -> Type;
    /// Gets the string value of this node. This will be `None` if the node is not a string node.
    fn string(&self) -> Option<&'a str>;
    /// Gets the integer value of this node. This will be `None` if the node is not an integer 
    /// node.
    fn integer(&self) -> Option<i64>;
    /// Gets the float value of this node. This will be `None` if the node is not a float node.
    fn float(&self) -> Option<f64>;
    /// Gets the vector value of this node. This will be `None` if the node is not a vector node.
    fn vector(&self) -> Option<(i32, i32)>;
}

/// A node in an NX file.
#[derive(Copy)]
pub struct Node<'a> {
    data: &'a Data,
    file: &'a super::File,
}

impl<'a> Node<'a> {
    /// Creates a Node from the data representing it and the file the data is from.
    pub unsafe fn construct(data: &'a Data, file: &'a super::File) -> Node<'a> {
        Node { data: data, file: file }
    }
    /// Gets whether or not the node is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.count == 0
    }
    /// Gets the name of this node from the string table.
    #[inline]
    pub fn name(&self) -> &'a str {
        self.file.get_str(self.data.name)
    }
    /// Gets an iterator over this node's children.
    #[inline]
    pub fn iter(&self) -> Nodes<'a> {
        let data = unsafe { self.file.nodetable.offset(self.data.children as isize) };
        Nodes {
            data: data,
            count: self.data.count,
            file: self.file,
        }
    }
}

impl<'a> GenericNode<'a> for Node<'a> {
    #[inline]
    fn get(&self, name: &str) -> Option<Node<'a>> {
        let mut data = unsafe { self.file.nodetable.offset(self.data.children as isize) };
        let mut count = self.data.count as isize;
        while count > 0 {
            let half = count / 2;
            let temp = unsafe { data.offset(half) };
            let other = self.file.get_str(unsafe { (*temp).name });
            match other.cmp(name) {
                Less => {
                    data = unsafe { temp.offset(1) };
                    count -= half + 1;
                },
                Equal => return Some(Node {
                    data: unsafe { &*temp },
                    file: self.file,
                }),
                Greater => count = half,
            }
        }
        None
    }
    #[inline]
    fn dtype(&self) -> Type {
        match FromPrimitive::from_u16(self.data.dtype) {
            Some(dtype) => dtype,
            None => Type::Empty,
        }
    }
    #[inline]
    fn string(&self) -> Option<&'a str> {
        match self.dtype() {
            Type::String => Some(self.file.get_str(unsafe {
                transmute::<_, String>(self.data.data).index
            })),
            _ => None,
        }
    }
    #[inline]
    fn integer(&self) -> Option<i64> {
        match self.dtype() {
            Type::Integer => Some(unsafe { transmute::<_, Integer>(self.data.data).value }),
            _ => None,
        }
    }
    #[inline]
    fn float(&self) -> Option<f64> {
        match self.dtype() {
            Type::Float => Some(unsafe { transmute::<_, Float>(self.data.data).value }),
            _ => None,
        }
    }
    #[inline]
    fn vector(&self) -> Option<(i32, i32)> {
        match self.dtype() {
            Type::Vector => Some(unsafe {
                let vec = transmute::<_, Vector>(self.data.data);
                (vec.x, vec.y)
            }),
            _ => None,
        }
    }
}
impl<'a> GenericNode<'a> for Option<Node<'a>> {
    #[inline]
    fn get(&self, name: &str) -> Option<Node<'a>> {
        match self {
            &Some(n) => n.get(name),
            &None => None,
        }
    }
    #[inline]
    fn dtype(&self) -> Type {
        match self {
            &Some(n) => n.dtype(),
            &None => Type::Empty,
        }
    }
    #[inline]
    fn string(&self) -> Option<&'a str> {
        match self {
            &Some(n) => n.string(),
            &None => None,
        }
    }
    #[inline]
    fn integer(&self) -> Option<i64> {
        match self {
            &Some(n) => n.integer(),
            &None => None,
        }
    }
    #[inline]
    fn float(&self) -> Option<f64> {
        match self {
            &Some(n) => n.float(),
            &None => None,
        }
    }
    #[inline]
    fn vector(&self) -> Option<(i32, i32)> {
        match self {
            &Some(n) => n.vector(),
            &None => None,
        }
    }
}

impl<'a> PartialEq for Node<'a> {
    #[inline]
    fn eq(&self, other: &Node) -> bool {
        self.data as *const Data == other.data as *const Data
    }
}

impl<'a> Eq for Node<'a> {}

/// An iterator over nodes.
pub struct Nodes<'a> {
    data: *const Data,
    count: u16,
    file: &'a super::File,
}

impl<'a> Iterator for Nodes<'a> {
    type Item = Node<'a>;
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count as usize, Some(self.count as usize))
    }
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

/// The data contained by an NX node.
#[repr(packed)]
pub struct Data {
    name: u32,
    children: u32,
    count: u16,
    dtype: u16,
    data: u64,
}

/// The types of NX nodes.
#[derive(FromPrimitive, PartialEq, Eq, Copy)]
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
struct Integer {
    value: i64,
}

#[repr(packed)]
struct Float {
    value: f64,
}

#[repr(packed)]
struct String {
    index: u32,
    _unused: u32,
}

#[repr(packed)]
struct Vector {
    x: i32,
    y: i32,
}

#[repr(packed)]
#[allow(dead_code)]
struct Bitmap {
    index: u32,
    width: u16,
    height: u16,
}

#[repr(packed)]
#[allow(dead_code)]
struct Audio {
    index: u32,
    length: u32,
}
