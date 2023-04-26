//! contains all important data structures

use im::Vector as ImVec;

/// Used at compile time to represent the stack state at runtime
pub type StackInfo = ImVec<DataInfo>;

/// Represents compile time information about a symbol. Does not contain
/// an ast-id, because a symbol is used in many places
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub stack_idx: usize,
    pub dtype: DataType,
}

/// Represents TypeInformation at compile time
#[derive(Debug, Clone)]
pub enum DataType {
    String,
    Int,
    Vec(Box<DataType>),
    Ref(Box<DataType>),
}

/// Holds information about data on the stack
#[derive(Debug, Clone)]
pub struct DataInfo {
    /// type of the data
    pub dtype: DataType,
    /// ast_id of the node that was responsible for the creation
    /// of the corresponding data.
    pub ast_id: usize,
}

pub mod data;
pub use data::*;

pub mod scopes;
pub use scopes::*;

pub mod intermediate_bytecode;
pub use intermediate_bytecode::*;

pub mod final_bytecode;
pub use final_bytecode::*;

pub mod ast;
pub use ast::*;

pub mod opcode;
pub use opcode::*;
