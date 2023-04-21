use proc_macros::ByteConvertible;
use serde::{Deserialize, Serialize};

/// Representing Opcodes, all variants must have zero or one member.
/// This way, it's possible to simply convert between an object and it's raw representation
#[derive(Debug, Clone, Copy, ByteConvertible, PartialEq)]
pub enum OpCode {
    /// The first usize is the index of the stack at which the executable name
    /// is stored, and the second usize is the index of the stack at which
    /// The vec with the arguments is stored
    Exec((DataRef, DataRef)),
    /// the usize tells StrCat how many DataRefs follow in the bytecode
    StrCat(usize),
    /// Just Data that will be used by the last command that came before it
    DataRef(DataRef),
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
            DataRef(d) => {
                d.offset_data_section_addr(offset);
            }
            Exit(_) | StrCat(_) => {}
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let input = OpCode::Exec((DataRef::StackIdx(12), DataRef::DataSectionIdx(14)));
        let bytes = input.to_bytes();
        let deserialized = unsafe { OpCode::from_ptr(bytes.as_ptr()) };
        assert!(deserialized == input);
    }

    #[test]
    fn test_size() {
        let input = OpCode::Exec((DataRef::StackIdx(12), DataRef::DataSectionIdx(14)));
        assert!(input.serialized_size() == 18);
    }
}
