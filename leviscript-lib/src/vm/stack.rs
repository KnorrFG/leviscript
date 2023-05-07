use crate::core::*;
use crate::vm::*;
use std::ops::{Deref, DerefMut};

/// type that is used at runtime to represent the stack
#[derive(Default)]
pub struct Stack(pub Vec<StackEntry>);

impl Stack {
    fn push_val_and_ref(&mut self, v: StackEntry) {
        self.push(v);
        let ptr: *const Value<RuntimeRef> = self.last().unwrap().get_value_ref().unwrap();
        self.push(StackEntry::Data(Data::Ref(RuntimeRef::HeapRef(ptr))));
    }

    pub fn push_entry(&mut self, v: StackEntry) {
        match v {
            StackEntry::Data(_) => {
                self.push(v);
            }
            StackEntry::Value(_) => {
                self.push_val_and_ref(v);
            }
        }
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
#[derive(Debug, Clone)]
pub enum StackEntry {
    Data(RuntimeData),
    Value(RuntimeValue),
}

impl From<RuntimeData> for StackEntry {
    fn from(value: RuntimeData) -> Self {
        StackEntry::Data(value)
    }
}

impl From<String> for StackEntry {
    fn from(value: String) -> Self {
        StackEntry::Value(Value::Str(value))
    }
}

impl StackEntry {
    fn get_value_ref(&self) -> Option<&RuntimeValue> {
        if let StackEntry::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
