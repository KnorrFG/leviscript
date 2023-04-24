//! This file defines the opcodes, and some utility types and functions.
//! There is a lot of code gen going on here by means of the OpCode derive-macro.
//! I consider the macro part of this crate, even though it is technically a sub-crate.
//! Therefore I liberally use types in the code generation, because I know they exist, instead
//! of somehow getting them into the macro. For an independent macro crate, this would be a no-go
//! but in this private scenario, I think it's fine. The vm::* imports are here because of the
//! codegen, btw

use proc_macros::OpCode;
use serde::{Deserialize, Serialize};

use crate::vm::*;

/// Representing Opcodes, all variants must have zero or one member.
/// This way, it's possible to simply convert between an object and it's raw representation
#[derive(Debug, Clone, Copy, OpCode, PartialEq)]
pub enum OpCode {
    /// The first usize is the index of the stack at which the executable name
    /// is stored, and the second usize is the index of the stack at which
    /// The vec with the arguments is stored
    Exec((DataRef, DataRef)),
    /// the usize tells StrCat how many DataRefs follow in the bytecode
    StrCat(usize),
    /// Just Data that will be used by the last command that came before it
    DataRef(DataRef),
    /// Push a Ref to another value onto the stack
    PushRefToStack(DataRef),
    /// Push an int to the stack
    PushIntToStack(i64),
    /// exits the program, returns the result
    Exit(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DataRef {
    StackIdx(usize),
    DataSectionIdx(usize),
}

impl DataRef {
    pub fn offset_data_section_addr(&mut self, offset: usize) {
        if let Self::DataSectionIdx(i) = self {
            *i += offset;
        }
    }
}

impl OpCode {
    pub fn offset_data_section_addr(&mut self, offset: usize) {
        use OpCode::*;
        match self {
            Exec((a, b)) => {
                a.offset_data_section_addr(offset);
                b.offset_data_section_addr(offset);
            }
            DataRef(d) | PushRefToStack(d) => {
                d.offset_data_section_addr(offset);
            }
            Exit(_) | StrCat(_) | PushIntToStack(_) => {}
        };
    }
}
