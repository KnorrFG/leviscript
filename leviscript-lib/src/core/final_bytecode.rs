use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::core::*;

/// Represents bytecode in it's final form
#[derive(Debug, Clone)]
pub struct FinalByteCode {
    pub text: Vec<u8>,
    pub data: Vec<Value>,
    pub header: FinalByteCodeHeader,
}

/// the header of the final bytecode
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinalByteCodeHeader {
    /// version of the crate that compiled this bytecode
    pub version: [u16; 3],
    /// the ast that was the compilation input
    pub ast: Block,
    /// has one entry for each opcode in text. Contains the ast id
    /// the nth entry contains the ast id of the nth opcode
    pub ast_ids: Vec<usize>,
    /// contains the mapping from offset in the byte-Vec to
    /// index of opcode in OpCode-Vec
    pub index: HashMap<usize, usize>,
}

impl FinalByteCode {
    /// get's the index of an opcode from it's address
    pub fn pc_to_index(&self, pc: *const u8) -> usize {
        let byte_offset = pc as usize - self.text.as_ptr() as usize;
        self.header.index[&byte_offset]
    }

    /// get's the id of the ast-node that was compiled into the opcode under
    /// the pc
    pub fn pc_to_ast_id(&self, pc: *const u8) -> usize {
        self.header.ast_ids[self.pc_to_index(pc)]
    }
}
