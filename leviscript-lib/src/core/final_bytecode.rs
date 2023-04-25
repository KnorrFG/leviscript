use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::core::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinalByteCode {
    pub text: Vec<u8>,
    pub data: Vec<Data>,
    pub header: FinalByteCodeHeader,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinalByteCodeHeader {
    pub version: [u16; 3],
    pub ast: Block,
    pub ast_ids: Vec<usize>,
    /// contains the mapping from offset in the byte-Vec to
    /// index of opcode in OpCode-Vec
    pub index: HashMap<usize, usize>,
}

impl FinalByteCode {
    pub fn pc_to_index(&self, pc: *const u8) -> usize {
        let byte_offset = pc as usize - self.text.as_ptr() as usize;
        self.header.index[&byte_offset]
    }

    pub fn pc_to_ast_id(&self, pc: *const u8) -> usize {
        self.header.ast_ids[self.pc_to_index(pc)]
    }
}
