use crate::vm::{type_error, Error, Result};
use std::result::Result as StdResult;

use crate::core::*;

/// used at runtime to represent all memory
pub struct Memory<'a> {
    /// the stack
    pub stack: Stack,
    /// the data section of the byte code
    pub data: &'a Vec<Data>,
}

/// type that is used at runtime to represent the stack
pub type Stack = Vec<StackEntry>;

/// A stack entry can be different things, one layer of indirection
/// for good measure
#[derive(Debug)]
pub enum StackEntry {
    FrameBorder,
    Entry(Data),
}

pub trait GetFromMemoryAs<'a>: Sized {
    fn get_from_mem(mem: &'a Memory<'a>, d: &'a Data) -> Result<Self>;
}

impl<'a> Memory<'a> {
    pub fn deref_refobj(&'a self, dref: &DataRef) -> Result<&'a Data> {
        use DataRef::*;
        match dref {
            StackIdx(i) => {
                if let StackEntry::Entry(e) = &self.stack[*i] {
                    Ok(e)
                } else {
                    Err(Error::UnexpectedStackEntry {
                        index: *i,
                        msg: format!("Expected Entry, found {:#?}", self.stack[*i]),
                        data_ref: *dref,
                    })
                }
            }
            DataSectionIdx(i) => Ok(&self.data[*i]),
        }
    }

    pub fn get_as<T: 'a + GetFromMemoryAs<'a>>(&'a self, d: &'a Data) -> Result<T> {
        <T as GetFromMemoryAs>::get_from_mem(self, d)
    }
}

impl<'a, T> GetFromMemoryAs<'a> for T
where
    T: 'a,
    PrimitiveValue: TryAsRef<'a, Self>,
{
    fn get_from_mem(mem: &'a Memory<'a>, d: &'a Data) -> Result<Self> {
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

impl<'a, T: 'a + GetFromMemoryAs<'a>> GetFromMemoryAs<'a> for Vec<T> {
    fn get_from_mem(mem: &'a Memory<'a>, d: &'a Data) -> Result<Self> {
        match d {
            Data::Vec(v) => {
                let res = v
                    .iter()
                    .map(|d| mem.get_as(d))
                    .collect::<StdResult<_, _>>()?;
                Ok(res)
            }
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

fn get_primitive_as<'a, T>(p: &'a PrimitiveValue) -> Result<T>
where
    T: 'a,
    PrimitiveValue: TryAsRef<'a, T>,
{
    p.try_as_ref()
        .ok_or_else(|| type_error::<T>(p.clone().into()))
}
