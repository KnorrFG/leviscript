//! Deals with types

use super::CopyValue;

/// Represents TypeInformation at compile time
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    HeapType(HeapType),
    StackType(StackType),
    Callable(CallableType, Box<Signature>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeapType {
    Str,
    Keyword,
    Vec(Box<DataType>),
    Dict(Box<DataType>, Box<DataType>),
    Set(Box<DataType>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallableType {
    Fn,
    Builtin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackType {
    Int,
    Float,
    Bool,
    Unit,
}

impl From<CopyValue> for StackType {
    fn from(value: CopyValue) -> Self {
        use CopyValue::*;
        match value {
            Int(_) => Self::Int,
            Float(_) => Self::Float,
            Bool(_) => Self::Bool,
            Unit => Self::Unit,
        }
    }
}

impl DataType {
    pub fn vec(self) -> Self {
        Self::HeapType(HeapType::Vec(Box::new(self)))
    }
    pub fn str() -> Self {
        Self::HeapType(HeapType::Str)
    }
    pub fn unit() -> Self {
        Self::StackType(StackType::Unit)
    }
    pub fn int() -> Self {
        Self::StackType(StackType::Int)
    }
}

/// represents a Function signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub args: Vec<DataType>,
    pub result: DataType,
    pub var_arg: Option<DataType>,
}

impl Signature {
    pub fn new() -> Self {
        Self {
            args: vec![],
            result: DataType::unit(),
            var_arg: None,
        }
    }

    pub fn args(self, args: Vec<DataType>) -> Self {
        Self { args, ..self }
    }

    pub fn arg(self, arg: DataType) -> Self {
        Self {
            args: vec![arg],
            ..self
        }
    }

    pub fn result(self, result: DataType) -> Self {
        Self { result, ..self }
    }

    pub fn variadic(self, var_arg: DataType) -> Self {
        Self {
            var_arg: Some(var_arg),
            ..self
        }
    }
}

impl DataType {
    /// if self is callible, returns the return type
    pub fn try_get_return_type(&self) -> Option<DataType> {
        match self {
            Self::Callable(_, sig) => Some(sig.result.clone()),
            _ => None,
        }
    }
}
