//! Deals with types

use super::CopyValue;

use std::collections::BTreeSet;

/// Represents TypeInformation at compile time
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    HeapType(HeapType),
    StackType(StackType),
    Callable(CallableType, Box<Signature>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeapType {
    Str,
    Keyword,
    Vec(Box<DataType>),
    Dict(Box<DataType>, Box<DataType>),
    Set(Box<DataType>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallableType {
    FnFragment,
    Builtin,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackType {
    Int,
    Float,
    Bool,
    Unit,
}

/// represents a Function signature
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Signature {
    pub args: Vec<TypeSet>,
    pub result: TypeSet,
    pub var_arg: Option<TypeSet>,
}

/// A template's main ability is to determine whether a type fits it.
///
/// Think of a template as a hole with a certain shape, and a type as an object that needs to fit
/// through that shape. AllTypes is the biggest Shape possible, everything fits through it, and a
/// concrete type is the most specifi shape possible, and only one type will fit it. However a
/// concrete type is just a special case of a TypeSet with one element, so that isn't covered
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeSet {
    //BTreeSet's dont' Require Hash, HashSets themselves don't
    //implement Hash, so I can't use them here
    SomeTypes(BTreeSet<DataType>),
    AllTypes,
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

impl From<HeapType> for DataType {
    fn from(value: HeapType) -> Self {
        Self::HeapType(value)
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

impl Signature {
    pub fn new() -> Self {
        Self {
            args: vec![],
            result: DataType::unit().into(),
            var_arg: None,
        }
    }

    pub fn args(self, args: Vec<TypeSet>) -> Self {
        Self { args, ..self }
    }

    pub fn arg(self, arg: TypeSet) -> Self {
        Self {
            args: vec![arg],
            ..self
        }
    }

    pub fn result(self, result: TypeSet) -> Self {
        Self { result, ..self }
    }

    pub fn variadic(self, var_arg: TypeSet) -> Self {
        Self {
            var_arg: Some(var_arg),
            ..self
        }
    }

    pub fn is_sattisfied_by(&self, args: &Vec<DataType>) -> bool {
        for (type_set, arg_type) in self.args.iter().zip(args.iter()) {
            if !type_set.is_sattisfied_by(arg_type) {
                return false;
            }
        }
        let mut iter = args.iter().skip(self.args.len());
        if let Some(var_arg_type_set) = &self.var_arg {
            // check that all remaining args sattisfy the var-arg type set
            iter.all(|arg_type| var_arg_type_set.is_sattisfied_by(arg_type))
        } else {
            // when there are no var args, the values are passing, if there are no more.
            iter.count() == 0
        }
    }

    pub fn get_nth_arg(&self, n: usize) -> Option<&TypeSet> {
        if n < self.args.len() {
            Some(&self.args[n])
        } else {
            // if we're here the result is the type of the vararg, if a vararg exists, otherwise
            // none. And that is the same logic as the following line
            self.var_arg.as_ref()
        }
    }
}

impl DataType {
    /// if self is callible, returns the return type
    pub fn try_get_return_type(&self) -> Option<TypeSet> {
        match self {
            Self::Callable(_, sig) => Some(sig.result.clone()),
            _ => None,
        }
    }
}

impl TypeSet {
    pub fn is_sattisfied_by(&self, t: &DataType) -> bool {
        match self {
            TypeSet::SomeTypes(ts) => ts.contains(&t),
            TypeSet::AllTypes => true,
        }
    }

    /// if the template represents a single type, it returns it, other wise None
    pub fn concrete_type(&self) -> Option<&DataType> {
        match self {
            TypeSet::SomeTypes(ts) => {
                if ts.len() == 1 {
                    ts.iter().next()
                } else {
                    None
                }
            }
            TypeSet::AllTypes => None,
        }
    }
}

impl From<DataType> for TypeSet {
    fn from(t: DataType) -> Self {
        Self::SomeTypes(BTreeSet::from([t]))
    }
}
