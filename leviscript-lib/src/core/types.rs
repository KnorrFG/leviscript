//! Deals with types

/// Represents TypeInformation at compile time
#[derive(Debug, Clone)]
pub enum DataType {
    Str,
    Int,
    Vec(Box<DataType>),
    Ref(Box<DataType>),
    Unit,
}

impl DataType {
    pub fn vec(self) -> Self {
        Self::Vec(Box::new(self))
    }
}

/// represents a Function signature
#[derive(Debug, Clone)]
pub struct Signature {
    pub args: Vec<DataType>,
    pub result: DataType,
}
