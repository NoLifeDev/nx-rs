// Copyright © 2015, Peter Atashian
//! Stuff for working with NX nodes

use std::cmp::Ordering::{Equal, Greater, Less};
use std::mem::{transmute};

use audio::{Audio};
use bitmap::{Bitmap};
use file::{File};
use repr;

pub use repr::Type;

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
    /// Gets the audio value of thise node. This will be `None` if the node is not an audio node.
    fn audio(&self) -> Option<Audio<'a>>;
    /// Gets the bitmap value of thise node. This will be `None` if the node is not a bitmap node.
    fn bitmap(&self) -> Option<Bitmap<'a>>;
}

/// A node in an NX file.
#[derive(Clone, Copy)]
pub struct Node<'a> {
    data: &'a repr::Node,
    file: &'a super::File,
}

impl<'a> Node<'a> {
    /// Creates a Node from the data representing it and the file the data is from.
    #[inline]
    pub unsafe fn construct(data: &'a repr::Node, file: &'a super::File) -> Node<'a> {
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
        unsafe { self.file.get_str(self.data.name) }
    }
    /// Gets an iterator over this node's children.
    #[inline]
    pub fn iter(&self) -> Nodes<'a> {
        let data = unsafe { self.file.get_node(self.data.children) };
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
        let mut data = unsafe { self.file.get_node(self.data.children) as *const repr::Node };
        let mut count = self.data.count as isize;
        while count > 0 {
            let half = count / 2;
            let temp = unsafe { data.offset(half) };
            let other = unsafe { self.file.get_str((*temp).name) };
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
        match self.data.dtype {
            0 => Type::Empty,
            1 => Type::Integer,
            2 => Type::Float,
            3 => Type::String,
            4 => Type::Vector,
            5 => Type::Bitmap,
            6 => Type::Audio,
            _ => Type::Empty
        }
    }
    #[inline]
    fn string(&self) -> Option<&'a str> {
        match self.dtype() {
            Type::String => Some(unsafe {
                self.file.get_str(transmute::<_, repr::String>(self.data.data).index)
            }),
            _ => None,
        }
    }
    #[inline]
    fn integer(&self) -> Option<i64> {
        match self.dtype() {
            Type::Integer => Some(unsafe { transmute::<_, repr::Integer>(self.data.data).value }),
            _ => None,
        }
    }
    #[inline]
    fn float(&self) -> Option<f64> {
        match self.dtype() {
            Type::Float => Some(unsafe { transmute::<_, repr::Float>(self.data.data).value }),
            _ => None,
        }
    }
    #[inline]
    fn vector(&self) -> Option<(i32, i32)> {
        match self.dtype() {
            Type::Vector => Some(unsafe {
                let vec = transmute::<_, repr::Vector>(self.data.data);
                (vec.x, vec.y)
            }),
            _ => None,
        }
    }
    #[inline]
    fn audio(&self) -> Option<Audio<'a>> {
        match self.dtype() {
            Type::Audio => Some(unsafe {
                let audio = transmute::<_, repr::Audio>(self.data.data);
                Audio::construct(self.file.get_audio(audio.index, audio.length))
            }),
            _ => None,
        }
    }
    #[inline]
    fn bitmap(&self) -> Option<Bitmap<'a>> {
        match self.dtype() {
            Type::Bitmap => Some(unsafe {
                let bitmap = transmute::<_, repr::Bitmap>(self.data.data);
                Bitmap::construct(self.file.get_bitmap(bitmap.index), bitmap.width, bitmap.height)
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
    #[inline]
    fn audio(&self) -> Option<Audio<'a>> {
        match self {
            &Some(n) => n.audio(),
            &None => None,
        }
    }
    #[inline]
    fn bitmap(&self) -> Option<Bitmap<'a>> {
        match self {
            &Some(n) => n.bitmap(),
            &None => None,
        }
    }
}

impl<'a> PartialEq for Node<'a> {
    #[inline]
    fn eq(&self, other: &Node) -> bool {
        self.data as *const repr::Node == other.data as *const repr::Node
    }
}

impl<'a> Eq for Node<'a> {}

/// An iterator over nodes.
pub struct Nodes<'a> {
    data: *const repr::Node,
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

