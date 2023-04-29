//! contains the exec functions that correspond to the [OpCode](crate::core::OpCode) variants

use crate::core::*;
use std::result::Result as StdResult;
use thiserror::Error;

pub mod built_ins;
pub mod heap;
pub mod memory;

pub use heap::*;
pub use memory::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error(
        "TypeError, tried to access:\n{accessed_data:#?} \n\
        as type {expected_type}"
    )]
    TypeError {
        accessed_data: RuntimeData,
        expected_type: &'static str,
    },

    #[error("Unexpected stack entry at {index}: {msg}")]
    UnexpectedStackEntry { index: usize, msg: String },

    #[error("Unexpected Opcode at {pc:?}, expected {expected}, found:\n{found}")]
    UnexpectedOpcode {
        pc: *const u8,
        expected: String,
        found: String,
    },

    #[error("Opcode at pc is Non-executable")]
    NonExecutableOpCode,

    #[error("The stack was empty unexpectedly: {0}")]
    StackEmpty(String),

    #[error("Unknown Builtin: {0}")]
    UnknownBuiltIn(String),
}

// fn type_error<T: ?Sized>(d: RuntimeData) -> Error {
//     Error::TypeError {
//         accessed_data: d,
//         expected_type: type_name::<T>(),
//     }
// }

/// returned by all exec_ functions
pub enum ExecOutcome {
    /// the new value for the program counter
    Pc(*const u8),
    /// All good, script finished successfull
    ExitCode(i32),
}

pub type Result<T> = StdResult<T, Error>;
pub type ExecResult = Result<ExecOutcome>;

macro_rules! rt_err{
    ($msg:literal $(, $args:expr)*) => { Error::Runtime(format!($msg $(, $args)*)) };
}
pub(crate) use rt_err;

macro_rules! rt_assert{
    ($cond:expr, $msg:literal $(, $args:expr)*) => {
        if ! $cond { return Err(rt_err!($msg $(, $args)*)); }
    };
}
pub(crate) use rt_assert;

// macro_rules! bail{
//     ($($err:tt)*) => {
//         return Err(Error::$($err)*);
//     };
// }

macro_rules! size_of {
    ($i:ident) => {
        OpCode::serialized_size_of(OpCode::$i)
    };
}

macro_rules! isize_of {
    ($u:ident) => {
        size_of!($u) as isize
    };
}

macro_rules! ok_pc {
    ($pc:expr) => {
        Ok(ExecOutcome::Pc($pc))
    };
}

pub unsafe fn exec_exec(pc: *const u8, mem: &mut Memory) -> ExecResult {
    built_ins::wrapper_1_var(built_ins::impls::exec, "exec", mem)?;
    ok_pc!(pc.offset(isize_of!(EXEC)))
}

pub unsafe fn exec_strcat(pc: *const u8, mem: &mut Memory) -> ExecResult {
    built_ins::wrapper_0_var_ret(built_ins::impls::strcat, "strcat", mem)?;
    ok_pc!(pc.offset(isize_of!(STRCAT)))
}

pub unsafe fn exec_exit(pc: *const u8, _: &mut Memory) -> ExecResult {
    let res = get_body!(Exit, pc.offset(2));
    Ok(ExecOutcome::ExitCode(*res))
}

pub unsafe fn exec_pushdatasecref(pc: *const u8, mem: &mut Memory) -> ExecResult {
    let dref = get_body!(PushDataSecRef, pc.offset(2));
    mem.push_data_section_ref(*dref);
    ok_pc!(pc.offset(isize_of!(PUSHDATASECREF)))
}

pub unsafe fn exec_pushprimitive(pc: *const u8, mem: &mut Memory) -> ExecResult {
    let res = get_body!(PushPrimitive, pc.offset(2));
    mem.push_stack(Data::CopyVal(*res));
    ok_pc!(pc.offset(isize_of!(PUSHPRIMITIVE)))
}

pub unsafe fn exec_repushstackentry(pc: *const u8, mem: &mut Memory) -> ExecResult {
    let idx = get_body!(RepushStackEntry, pc.offset(2));
    mem.copy_stack_entry_to_top(*idx);
    ok_pc!(pc.offset(isize_of!(REPUSHSTACKENTRY)))
}
