//! Deals with run-time data representation

//! I want data access to be fast, I don't want unneccessary Rc increments and decrements,
//! since the compiler knows when stuff dies, and I don't want to clone heap data like strings
//! or vecs, so this means we need pointers. When an expensive type (i.e. all non copy types) is
//! created, the value is put on the stack, and then a ref to that value is put on top of it.
//! So all further calls work with the ref.
//!
//! During compilation, the compiler will know the scope of a value that a ref points to,
//! and make sure to return the value too, if a ref to value of the current scopr is returned

use im::{HashMap, HashSet, Vector};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

pub trait RefRequirements: Debug + Clone + PartialEq + Eq + Hash {}

// This is where non-copy values are stored. It lives on the stack, but it is not an element of
// Data, as Data should be Copy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value<RefT: RefRequirements> {
    Str(String),
    Keyword(String),
    Vec(Vector<Data<RefT>>),
    Dict(HashMap<Data<RefT>, Data<RefT>>),
    Set(HashSet<Data<RefT>>),
}

pub type ComptimeValue = Value<()>;
pub type RuntimeValue = Value<RuntimeRef>;

// Holds copy values
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CopyValue {
    Int(i64),
    Float(OrderedFloat<f64>),
    Bool(bool),
    Unit,
}

/// Represents all possible values.
///
/// That means either a real value or a reference to a value.
/// The problem is, that at runtime, a reference is a pointer, which can't
/// exist at compile time. So at compile time, references are indices
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Data<RefT> {
    CopyVal(CopyValue),
    Ref(RefT),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeRef {
    HeapRef(*const Value<Self>),
    DataSecRef(*const Value<()>),
}

impl RefRequirements for RuntimeRef {}
impl RefRequirements for () {}

pub type ComptimeData = Data<()>;
pub type RuntimeData = Data<RuntimeRef>;

pub trait TryFromRef<SrcT>: Sized {
    unsafe fn try_from_ref(s: &SrcT) -> Option<Self>;
}

pub trait TryIntoRef<T> {
    unsafe fn rtry_into(&self) -> Option<T>;
}

// ==============================================================================
// Impls
// ==============================================================================
impl<Implementor, T: TryFromRef<Implementor>> TryIntoRef<T> for Implementor {
    unsafe fn rtry_into(&self) -> Option<T> {
        T::try_from_ref(self)
    }
}

// ==============================================================================
// Copy Value
// ==============================================================================
impl From<i64> for CopyValue {
    fn from(x: i64) -> Self {
        CopyValue::Int(x)
    }
}

impl From<u64> for CopyValue {
    fn from(x: u64) -> Self {
        CopyValue::Int(x as i64)
    }
}

impl From<usize> for CopyValue {
    fn from(x: usize) -> Self {
        CopyValue::Int(x as i64)
    }
}

impl From<isize> for CopyValue {
    fn from(x: isize) -> Self {
        CopyValue::Int(x as i64)
    }
}

impl From<bool> for CopyValue {
    fn from(x: bool) -> Self {
        CopyValue::Bool(x)
    }
}

impl From<f64> for CopyValue {
    fn from(x: f64) -> Self {
        CopyValue::Float(OrderedFloat(x))
    }
}

impl From<()> for CopyValue {
    fn from(x: ()) -> Self {
        CopyValue::Unit
    }
}

impl TryFrom<CopyValue> for i64 {
    type Error = ();
    fn try_from(v: CopyValue) -> Result<i64, ()> {
        if let CopyValue::Int(i) = v {
            Ok(i)
        } else {
            Err(())
        }
    }
}

impl TryFrom<CopyValue> for f64 {
    type Error = ();
    fn try_from(v: CopyValue) -> Result<f64, ()> {
        if let CopyValue::Float(x) = v {
            Ok(x.0)
        } else {
            Err(())
        }
    }
}

impl TryFrom<CopyValue> for bool {
    type Error = ();
    fn try_from(v: CopyValue) -> Result<bool, ()> {
        if let CopyValue::Bool(x) = v {
            Ok(x)
        } else {
            Err(())
        }
    }
}

impl TryFrom<CopyValue> for () {
    type Error = ();
    fn try_from(v: CopyValue) -> Result<(), ()> {
        if let CopyValue::Unit = v {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl TryFrom<CopyValue> for *const String {
    type Error = ();
    fn try_from(value: CopyValue) -> Result<Self, Self::Error> {
        Err(())
    }
}

impl<T: TryFrom<CopyValue>> TryFromRef<CopyValue> for T {
    unsafe fn try_from_ref(s: &CopyValue) -> Option<Self> {
        (*s).try_into().ok()
    }
}

// ==============================================================================
// TryFromRef<Value<RefT>>
// ==============================================================================
impl<RefT: RefRequirements> TryFromRef<Value<RefT>> for *const String {
    unsafe fn try_from_ref(s: &Value<RefT>) -> Option<Self> {
        match s {
            Value::Str(s) => Some(s),
            Value::Keyword(kw) => Some(kw),
            _ => None,
        }
    }
}

impl<RefT, InnerT> TryFromRef<Value<RefT>> for Vec<InnerT>
where
    RefT: RefRequirements,
    InnerT: TryFromRef<Data<RefT>>,
{
    unsafe fn try_from_ref(s: &Value<RefT>) -> Option<Self> {
        match s {
            Value::Vec(s) => s.iter().map(|e| e.rtry_into()).collect::<Option<_>>(),
            _ => None,
        }
    }
}

// ==============================================================================
// TryFromRef<RefT>
// ==============================================================================
impl<T> TryFromRef<()> for T {
    unsafe fn try_from_ref(_: &()) -> Option<Self> {
        None
    }
}

impl<T> TryFromRef<RuntimeRef> for T
where
    T: TryFromRef<Value<RuntimeRef>> + TryFromRef<Value<()>>,
{
    unsafe fn try_from_ref(s: &RuntimeRef) -> Option<Self> {
        match s {
            RuntimeRef::HeapRef(r) => (**r).rtry_into(),
            RuntimeRef::DataSecRef(r) => (**r).rtry_into(),
        }
    }
}

// i wanted to to it for all T: CopyValue, but for some reason that conflicts with the above definition
// even though Copy val is implemented for: i64, f64, bool, ()
// and TryFromRef<Value<RuntimeRef>> is implemented for *const String and Vec<T>
impl TryFromRef<RuntimeRef> for i64 {
    unsafe fn try_from_ref(_: &RuntimeRef) -> Option<Self> {
        None
    }
}

// ==============================================================================
// TryFromRef<Data<RefT>>
// ==============================================================================
impl<RefT, TargetT: TryFromRef<RefT> + TryFromRef<CopyValue>> TryFromRef<Data<RefT>> for TargetT {
    unsafe fn try_from_ref(s: &Data<RefT>) -> Option<Self> {
        match s {
            Data::CopyVal(v) => v.rtry_into(),
            Data::Ref(r) => r.rtry_into(),
        }
    }
}

// ==============================================================================
// Simple Conversions
// ==============================================================================
impl<T> From<CopyValue> for Data<T> {
    fn from(value: CopyValue) -> Self {
        Self::CopyVal(value)
    }
}

impl From<RuntimeRef> for RuntimeData {
    fn from(value: RuntimeRef) -> Self {
        Self::Ref(value)
    }
}

impl<RefT> From<String> for Value<RefT>
where
    RefT: RefRequirements,
{
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}
