use im::Vector as ImVec;

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub stack_idx: usize,
    pub dtype: DataType,
}

#[derive(Debug, Clone)]
pub enum DataType {
    String,
    Int,
    Vec(Box<DataType>),
    Ref(Box<DataType>),
}

/// Used at compile time to represent the stack state at runtime
pub type StackInfo = ImVec<DataInfo>;

#[derive(Debug, Clone)]
pub struct DataInfo {
    pub dtype: DataType,
    pub ast_id: usize,
}

mod data;
pub use data::*;

mod scopes;
pub use scopes::*;

mod intermediate_bytecode;
pub use intermediate_bytecode::*;

mod final_bytecode;
pub use final_bytecode::*;

mod ast;
pub use ast::*;

mod opcode;
pub use opcode::*;
