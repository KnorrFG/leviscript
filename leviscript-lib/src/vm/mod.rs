//! contains the exec functions that correspond to the [OpCode](crate::core::OpCode) variants

use crate::core::*;
use std::result::Result as StdResult;
use std::{any::type_name, process};
use thiserror::Error;

mod memory;
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
        accessed_data: Data,
        expected_type: &'static str,
    },

    #[error("Unexpected stack entry at {index} via {data_ref:?}: {msg}")]
    UnexpectedStackEntry {
        index: usize,
        msg: String,
        data_ref: DataRef,
    },

    #[error("Unexpected Opcode at {pc:?}, expected {expected}, found:\n{found}")]
    UnexpectedOpcode {
        pc: *const u8,
        expected: String,
        found: String,
    },

    #[error("Opcode at pc is Non-executable")]
    NonExecutableOpCode,
}

fn type_error<T: ?Sized>(d: Data) -> Error {
    Error::TypeError {
        accessed_data: d,
        expected_type: type_name::<T>(),
    }
}

/// returned by all exec_ functions
pub enum ExecOutcome {
    // the new value for the program counter
    Pc(*const u8),
    // All good, script finished successfull
    ExitCode(i32),
}

pub type Result<T> = StdResult<T, Error>;
pub type ExecResult = Result<ExecOutcome>;

macro_rules! rt_err{
    ($msg:literal $(, $args:expr)*) => { Error::Runtime(format!($msg $(, $args)*)) };
}

macro_rules! rt_assert{
    ($cond:expr, $msg:literal $(, $args:expr)*) => {
        if ! $cond { return Err(rt_err!($msg $(, $args)*)); }
    };
}

macro_rules! bail{
    ($($err:tt)*) => {
        return Err(Error::$($err)*);
    };
}

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
    let (bin, args) = get_body!(Exec, pc.offset(2));
    let bin_name = mem.get_as::<&str>(mem.deref_refobj(bin)?)?;
    let resolved_args = mem.deref_refobj(&args)?;
    let Data::Vec(args) = resolved_args else {
         bail!(TypeError { accessed_data: resolved_args.clone(), expected_type: "Vec"});
    };
    let args_as_str: Vec<&str> = args
        .iter()
        .map(|arg| mem.get_as::<&str>(arg))
        .collect::<StdResult<_, _>>()?;
    let stat = process::Command::new(&bin_name)
        .args(&args_as_str)
        .status()
        .map_err(|e| rt_err!("Executing {}: {}", bin_name, e))?;
    rt_assert!(stat.success(), "{} did not execute successfully", bin_name);
    ok_pc!(pc.offset(isize_of!(EXEC)))
}

pub unsafe fn exec_strcat(pc: *const u8, mem: &mut Memory) -> ExecResult {
    let n = get_body!(StrCat, pc.offset(2));
    let n_is = *n as isize;
    let elems: Vec<&str> = (0isize..n_is)
        .map(|i| {
            let addr = pc.offset(isize_of!(STRCAT) + i * isize_of!(DATAREF) + 2);
            mem.get_as(mem.deref_refobj(get_body!(DataRef, addr))?)
        })
        .collect::<StdResult<_, _>>()?;
    mem.stack.push(StackEntry::Entry(elems.join("").into()));
    ok_pc!(pc.offset(isize_of!(STRCAT) + n_is * isize_of!(DATAREF)))
}

pub unsafe fn exec_dataref(_: *const u8, _: &mut Memory) -> ExecResult {
    Err(Error::NonExecutableOpCode)
}

pub unsafe fn exec_exit(pc: *const u8, _: &mut Memory) -> ExecResult {
    let res = get_body!(Exit, pc.offset(2));
    Ok(ExecOutcome::ExitCode(*res))
}

pub unsafe fn exec_pushreftostack(pc: *const u8, mem: &mut Memory) -> ExecResult {
    let idx = get_body!(PushRefToStack, pc.offset(2));
    mem.stack.push(StackEntry::Entry(Data::Ref(*idx)));
    ok_pc!(pc.offset(isize_of!(PUSHREFTOSTACK)))
}

pub unsafe fn exec_pushinttostack(pc: *const u8, mem: &mut Memory) -> ExecResult {
    let idx = get_body!(PushIntToStack, pc.offset(2));
    mem.stack.push(StackEntry::Entry((*idx).into()));
    ok_pc!(pc.offset(isize_of!(PUSHINTTOSTACK)))
}
