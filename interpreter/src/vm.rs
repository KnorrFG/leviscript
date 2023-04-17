use crate::{
    bytecode::{self, Data, DataAs},
    opcode::{DataRef, OpCode},
    utils,
};
use anyhow::{ensure, Context};
use std::process;

pub struct VmState {
    pub pc: *const u8,
    pub memory: Memory,
}

pub struct Memory {
    pub stack: Vec<StackEntry>,
    pub data: Vec<Data>,
}

pub enum StackEntry {
    FrameBorder,
    Entry(Data),
}

pub fn run(bc: bytecode::Final) -> anyhow::Result<i32> {
    let mut state = VmState {
        memory: Memory {
            stack: vec![],
            data: bc.data,
        },
        pc: bc.text.as_ptr(),
    };

    loop {
        unsafe {
            let opcode = OpCode::from_ptr(state.pc);
            use OpCode::*;
            state.pc = match opcode {
                Exec((bin, args)) => {
                    let bin_name = state.memory.get_as::<&str>(bin);
                    let args = state.memory.get_as::<Vec<&str>>(args);
                    let stat = process::Command::new(bin_name)
                        .args(args)
                        .status()
                        .context(format!("Executing {}", bin_name))?;
                    ensure!(stat.success(), "{} did not execute successfully", bin_name);
                    state.pc.offset(opcode.serialized_size() as isize)
                }
                Exit(res) => return Ok(res),
            };
        }
    }
}

impl Memory {
    pub fn get_as<'a, T: DataAs<'a>>(&'a self, reference: DataRef) -> T {
        use DataRef::*;
        match reference {
            StackIdx(i) => {
                if let StackEntry::Entry(e) = &self.stack[i] {
                    e.get_as()
                } else {
                    utils::bug!("A DataRef pointed to a FrameBorder");
                }
            }
            DataSectionIdx(i) => self.data[i].get_as(),
        }
    }
}
