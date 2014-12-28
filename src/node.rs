// Copyright © 2014, Peter Atashian

use std::mem::transmute;
use std::num::FromPrimitive;

pub trait GenericNode<'a> {
    fn get(&self, name: &str) -> Option<Node<'a>>;
    fn dtype(&self) -> Type;
    fn string(&self) -> Option<&'a str>;
    fn integer(&self) -> Option<i64>;
    fn float(&self) -> Option<f64>;
    fn vector(&self) -> Option<(i32, i32)>;
}

#[deriving(Copy)]
pub struct Node<'a> {
    data: &'a Data,
    file: &'a super::File,
}

impl<'a> Node<'a> {
    pub unsafe fn construct(data: &'a Data, file: &'a super::File) -> Node<'a> {
        Node { data: data, file: file }
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.count == 0
    }
    #[inline]
    pub fn name(&self) -> &'a str {
        self.file.get_str(self.data.name)
    }
    #[inline]
    pub fn iter(&self) -> Nodes<'a> {
        let data = unsafe { self.file.nodetable.offset(self.data.children as int) };
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
        let mut data = unsafe { self.file.nodetable.offset(self.data.children as int) };
        let mut count = self.data.count as int;
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

pub struct Nodes<'a> {
    data: *const Data,
    count: u16,
    file: &'a super::File,
}

impl<'a> Iterator<Node<'a>> for Nodes<'a> {
    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.count as uint, Some(self.count as uint))
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

#[repr(packed)]
pub struct Data {
    name: u32,
    children: u32,
    count: u16,
    dtype: u16,
    data: u64,
}

#[deriving(FromPrimitive, PartialEq, Eq, Copy)]
pub enum Type {
    Empty = 0,
    Integer = 1,
    Float = 2,
    String = 3,
    Vector = 4,
    Bitmap = 5,
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
