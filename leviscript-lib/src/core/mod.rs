//! contains all important data structures

mod data;
pub use data::*;

mod scopes;
pub use scopes::*;

mod bytecode_builder;
pub use bytecode_builder::*;

mod bytecode;
pub use bytecode::*;

mod ast;
pub use ast::*;

mod opcode;
pub use opcode::*;

mod types;
pub use types::*;

mod heap;
pub use heap::*;
