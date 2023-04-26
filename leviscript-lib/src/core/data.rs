//! Deals with run-time data representation

use enum_variant_macros::{FromVariants, TryFromVariants};
use serde::{Deserialize, Serialize};
use strum_macros::IntoStaticStr;

/// Trait For Data. Used to get a ref to the contained
/// data, if expected type and variant match
pub trait TryAsRef<'a, T: 'a> {
    fn try_as_ref(&'a self) -> Option<T>;
}

impl<'a> TryAsRef<'a, &'a str> for PrimitiveValue {
    fn try_as_ref(&'a self) -> Option<&'a str> {
        match self {
            PrimitiveValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl<'a> TryAsRef<'a, &'a i64> for PrimitiveValue {
    fn try_as_ref(&self) -> Option<&i64> {
        match self {
            PrimitiveValue::Int(i) => Some(i),
            _ => None,
        }
    }
}

/// Represents simple values
#[derive(IntoStaticStr, FromVariants, TryFromVariants, Debug, Serialize, Deserialize, Clone)]
pub enum PrimitiveValue {
    String(String),
    Int(i64),
}

/// Represents all possible values
#[derive(IntoStaticStr, TryFromVariants, Debug, Serialize, Deserialize, Clone)]
pub enum Data {
    Primitive(PrimitiveValue),
    Vec(Vec<Data>),
    Ref(DataRef),
}

impl<T> From<T> for Data
where
    PrimitiveValue: From<T>,
{
    fn from(x: T) -> Self {
        Data::Primitive(x.into())
    }
}

/// represents a reference to a value. Can either live on the stack,
/// or in the data segment
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DataRef {
    StackIdx(usize),
    DataSectionIdx(usize),
}

impl Data {
    /// patch datasection addresses, if one is contained
    pub fn offset_data_section_addr(&mut self, offset: usize) {
        match self {
            Data::Vec(vec) => {
                for d in vec {
                    d.offset_data_section_addr(offset);
                }
            }
            Data::Ref(r) => r.offset_data_section_addr(offset),
            Data::Primitive(_) => {}
        }
    }
}

impl DataRef {
    /// patch datasection addresses, if one is contained
    pub fn offset_data_section_addr(&mut self, offset: usize) {
        if let Self::DataSectionIdx(i) = self {
            *i += offset;
        }
    }
}
