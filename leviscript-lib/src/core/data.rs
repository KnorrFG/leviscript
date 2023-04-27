//! Deals with run-time data representation

//! I want data access to be fast, I don't want unneccessary Rc increments and decrements,
//! since the compiler knows when stuff dies, and I don't want to clone heap data like strings
//! or vecs, so this means we need pointers. When an expensive type (i.e. all non copy types) is
//! created, the value is put on the stack, and then a ref to that value is put on top of it.
//! So all further calls work with the ref.
//!
//! During compilation, the compiler will know the scope of a value that a ref points to,
//! and make sure to return the value too, if a ref to value of the current scopr is returned

use enum_variant_macros::FromVariants;
use im::{HashMap, HashSet, Vector};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use ordered_float::OrderedFloat;
use strum_macros::IntoStaticStr;

use crate::vm::StackEntry;

// This is where non-copy values are stored. It lives on the stack, but it is not an element of
// Data, as Data should be Copy.
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Keyword(String),
    Vec(Vector<Data>),
    Dict(HashMap<Data, Data>),
    Set(HashSet<Data>),
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Str(v)
    }
}

// Holds copy values
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CopyValue {
    Int(i64),
    Float(OrderedFloat<f64>),
    Bool(bool),
    Unit,
}

/// Represents all possible values
#[derive(IntoStaticStr, FromVariants, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Data {
    CopyVal(CopyValue),
    Ref(*const Value),
}

pub trait GetDataAs<T> {
    unsafe fn get_as(&self) -> Option<T>;
}

macro_rules! impl_get_as_for_copy_val {
    ($ty:ty => $($arms:tt)*) => {
        impl GetDataAs<$ty> for Data {
            unsafe fn get_as(&self) -> Option<$ty>
            {
                match self {
                    $($arms)*
                    _ => None,
                }
            }
        }
    };
}

impl_get_as_for_copy_val!(i64 => Data::CopyVal(CopyValue::Int(i)) => Some(*i),);
impl_get_as_for_copy_val!(f64 => Data::CopyVal(CopyValue::Float(OrderedFloat(f))) => Some(*f),);
impl_get_as_for_copy_val!(bool => Data::CopyVal(CopyValue::Bool(b)) => Some(*b),);

impl GetDataAs<*const String> for Data {
    unsafe fn get_as(&self) -> Option<*const String> {
        if let Data::Ref(ptr) = self {
            let val_ref = &**ptr;
            match val_ref {
                Value::Str(s) => Some(s),
                Value::Keyword(k) => Some(k),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl<T> GetDataAs<Vec<T>> for Data
where
    Data: GetDataAs<T>,
{
    unsafe fn get_as(&self) -> Option<Vec<T>> {
        if let Data::Ref(ptr) = self {
            match &**ptr {
                Value::Vec(v) => v.iter().map(|d| d.get_as()).collect(),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl TryFrom<StackEntry> for Data {
    type Error = ();
    fn try_from(value: StackEntry) -> Result<Self, Self::Error> {
        if let StackEntry::Data(d) = value {
            Ok(d)
        } else {
            Err(())
        }
    }
}
