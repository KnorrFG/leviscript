//! Deals with memory access at runtime
//!
//! The Core Issue that this file deals with is the fact, that Data is
//! a recursive type that cannot resolve itself. I.e. if you have a
//! Data::Ref and want an Int, you need to go to memory to find out, what the value is.
//! A PrimitiveValue can be resolved by the value itself (using [`crate::core::TryAsRef`])
//! But Data cannot, therefore we go through [`Memory::get_as`], which uses the [`GetFromMemoryAs`]
//! trait, which is the way rust deals with return type overloading, for more details see
//! [my blog post on this topic](https://felix-knorr.net/posts/2023-04-17-traits.html)
//! in principle it's simple, there is one implementation that just forwards to PrimitiveValue
//! and one that handles vec. The tricky stuff is to get the lifetimes right, and when
//! to use T or &T

use crate::vm::{type_error, Error, Result};
use std::result::Result as StdResult;

use crate::core::*;

/// used at runtime to represent all memory
pub struct Memory<'a, 'b> {
    /// the stack
    pub stack: Stack,
    /// the data section of the byte code
    pub data: &'a Vec<Data<'b>>,
}

/// type that is used at runtime to represent the stack
pub type Stack<'a> = Vec<StackEntry<'a>>;

/// A stack entry can be different things, one layer of indirection
/// for good measure
#[derive(Debug)]
pub enum StackEntry<'a> {
    Data(Data<'a>),
    Value(Value<'a>),
}

impl<'mem> Memory<'mem> {
    /// returns a &Data for a DataRef
    pub fn deref_refobj(&'mem self, dref: &DataRef) -> Result<&'mem Data> {
        use DataRef::*;
        match dref {
            StackIdx(i) => {
                if let StackEntry::Entry(e) = &self.stack[*i] {
                    Ok(e)
                } else {
                    Err(Error::UnexpectedStackEntry {
                        index: *i,
                        msg: format!("Expected Entry, found {:#?}", self.stack[*i]),
                    })
                }
            }
            DataSectionIdx(i) => Ok(&self.data[*i]),
        }
    }

    /// get the content of a &Data. Errors if the returntype doesn't match the data
    pub fn get_as<'d, 'r, T>(&'mem self, d: &'d Data) -> Result<T>
    where
        T: GetFromMemoryAs<'r>,
        'mem: 'd,
        'd: 'r,
    {
        <T as GetFromMemoryAs>::get_from_mem(self, d)
    }
}

impl<'r, T> GetFromMemoryAs<'r> for T
where
    T: 'r,
    PrimitiveValue: TryAsRef<'r, T>,
{
    /// The implementation that deals with the primitive values, and
    /// looks into `Data::Ref`, if one is found
    fn get_from_mem<'m, 'd>(mem: &'m Memory, d: &'d Data) -> Result<Self>
    where
        'm: 'd,
        'd: 'r,
    {
        match d {
            Data::Primitive(p) => get_primitive_as(p),
            Data::Ref(r) => {
                let d = mem.deref_refobj(r)?;
                mem.get_as(d)
            }
            Data::Vec(_) => Err(type_error::<Self>(d.clone())),
        }
    }
}

impl<'r, T> GetFromMemoryAs<'r> for Vec<T>
where
    T: GetFromMemoryAs<'r>,
{
    /// The implementation for vectors
    fn get_from_mem<'m, 'd>(mem: &'m Memory, d: &'d Data) -> Result<Self>
    where
        'd: 'r,
        'm: 'd,
    {
        match d {
            Data::Vec(v) => {
                let res = v
                    .iter()
                    .map(|d| mem.get_as(d))
                    .collect::<StdResult<_, _>>()?;
                Ok(res)
            }
            Data::Ref(r) => mem.get_as(mem.deref_refobj(r)?),
            _ => Err(type_error::<Self>(d.clone())),
        }
    }
}

impl std::fmt::Display for StackEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            StackEntry::FrameBorder => "FrameBorder".into(),
            StackEntry::Entry(e) => format!("{:?}", e),
        };
        write!(f, "{}", s)
    }
}

fn get_primitive_as<'res, T: 'res>(p: &'res PrimitiveValue) -> Result<T>
where
    PrimitiveValue: TryAsRef<'res, T>,
{
    p.try_as_ref()
        .ok_or_else(|| type_error::<T>(p.clone().into()))
}
