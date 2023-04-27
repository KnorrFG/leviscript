//! contains the exec functions that correspond to the [OpCode](crate::core::OpCode) variants

use crate::core::*;
use std::any::type_name;
use std::ops::{Deref, DerefMut};
use std::result::Result as StdResult;
use thiserror::Error;

/// type that is used at runtime to represent the stack
pub struct Stack(Vec<StackEntry>);

impl Stack {
    pub fn push_val(&mut self, v: Value) {
        self.0.push(StackEntry::Value(v));
    }

    pub fn push_val_and_ref(&mut self, v: Value) {
        self.push_val(v);
        self.0.push(StackEntry::Data(Data::Ref(
            self.0.last().unwrap().get_value_ref().unwrap(),
        )));
    }
}

impl Deref for Stack {
    type Target = Vec<StackEntry>;
    fn deref(&self) -> &Vec<StackEntry> {
        &self.0
    }
}

impl DerefMut for Stack {
    fn deref_mut(&mut self) -> &mut Vec<StackEntry> {
        &mut self.0
    }
}
/// A stack entry can be different things, one layer of indirection
/// for good measure
#[derive(Debug)]
pub enum StackEntry {
    Data(Data),
    Value(Value),
}

impl From<Data> for StackEntry {
    fn from(value: Data) -> Self {
        StackEntry::Data(value)
    }
}

impl StackEntry {
    fn get_value_ref(&self) -> Option<&Value> {
        if let StackEntry::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
pub mod built_ins;

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

fn type_error<T: ?Sized>(d: Data) -> Error {
    Error::TypeError {
        accessed_data: d,
        expected_type: type_name::<T>(),
    }
}

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

pub unsafe fn exec_exec(pc: *const u8, stack: &mut Stack, data: &Vec<Value>) -> ExecResult {
    built_ins::wrapper_2(built_ins::impls::exec, "exec", stack)?;
    ok_pc!(pc.offset(isize_of!(EXEC)))
}

pub unsafe fn exec_strcat(pc: *const u8, stack: &mut Stack, data: &Vec<Value>) -> ExecResult {
    built_ins::wrapper_1_ret(built_ins::impls::strcat, "strcat", stack);
    ok_pc!(pc.offset(isize_of!(STRCAT)))
}

pub unsafe fn exec_exit(pc: *const u8, _: &mut Stack, data: &Vec<Value>) -> ExecResult {
    let res = get_body!(Exit, pc.offset(2));
    Ok(ExecOutcome::ExitCode(*res))
}

pub unsafe fn exec_pushdatasecref(
    pc: *const u8,
    stack: &mut Stack,
    data: &Vec<Value>,
) -> ExecResult {
    let dref = get_body!(PushDataSecRef, pc.offset(2));
    stack.push(Data::Ref(data.get_unchecked(dref.0)).into());
    ok_pc!(pc.offset(isize_of!(PUSHDATASECREF)))
}

pub unsafe fn exec_pushprimitive(
    pc: *const u8,
    stack: &mut Stack,
    data: &Vec<Value>,
) -> ExecResult {
    let res = get_body!(PushPrimitive, pc.offset(2));
    stack.push(Data::from(*res).into());
    ok_pc!(pc.offset(isize_of!(PUSHPRIMITIVE)))
}
