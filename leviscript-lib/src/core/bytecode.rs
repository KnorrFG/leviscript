use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::core::*;
use crate::vm;

/// Represents bytecode in it's final form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteCode {
    pub text: Vec<u8>,
    pub data: Vec<ComptimeValue>,
}

/// the header of the final bytecode
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugInformation {
    /// Has one entry for each opcode in text. The nth entry contains the ast id of the node
    /// that produced the corresponding opcode
    pub ast_ids: Vec<usize>,
    /// contains the mapping from offset in the byte-Vec to
    /// index of opcode in OpCode-Vec
    pub index: HashMap<usize, usize>,
}

/// Utility type that can be used to execute the bytecode
///
/// This struct is self referential. That means if you move it, the pointer becomes invalid.
/// Since calling new() actually moves the value, you need to call init() after creating it.
pub struct Runner {
    pub text: Vec<u8>,
    pub mem: vm::Memory,
    pub pc: *const u8,
}

pub enum StepResult {
    Ok,
    Done(i32),
    Err(vm::Error),
}

impl Runner {
    /// creates a new instance. That instance is not valid for use yet, you must call init first
    pub fn new(ByteCode { text, data }: ByteCode) -> Self {
        Runner {
            text,
            mem: data.into(),
            pc: std::ptr::null(),
        }
    }

    /// empties memory and sets the pc to the first instruction in the bytecode
    pub fn reset_pc(&mut self) {
        self.pc = self.text.as_ptr();
    }

    /// executes the current byte code instruction and adjustst the pc
    /// will deref a null pointer if init wasn't called yet
    pub unsafe fn step(&mut self) -> StepResult {
        let disc_ptr = self.pc as *const u16;
        use vm::ExecOutcome::*;
        match OpCode::dispatch_discriminant(*disc_ptr, self.pc, &mut self.mem) {
            Ok(Pc(new_pc)) => {
                self.pc = new_pc;
                StepResult::Ok
            }
            Ok(ExitCode(res)) => StepResult::Done(res),
            Err(e) => StepResult::Err(e),
        }
    }

    /// get's the index of the opcode that the pc points to currently
    pub fn current_bc_index(&self, info: &DebugInformation) -> usize {
        let byte_offset = self.pc as usize - self.text.as_ptr() as usize;
        info.index[&byte_offset]
    }

    /// get's the id of the ast-node that was compiled into the opcode under
    /// the pc
    pub fn pc_to_ast_id(&self, info: &DebugInformation) -> usize {
        info.ast_ids[self.current_bc_index(info)]
    }
}
