use proc_macros::ByteConvertible;

/// Representing Opcodes, all variants must have zero or one member.
/// This way, it's possible to simply convert between an object and it's raw representation
#[derive(Debug, Clone, Copy, ByteConvertible, PartialEq)]
pub enum OpCode {
    /// The first usize is the index of the stack at which the executable name
    /// is stored, and the second usize is the index of the stack at which
    /// The vec with the arguments is stored
    Exec((usize, usize)),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let input = OpCode::Exec((12, 14));
        let bytes = input.to_bytes();
        let deserialized = unsafe { OpCode::from_ptr(bytes.as_ptr()) };
        assert!(deserialized == input);
    }

    #[test]
    fn test_size() {
        let input = OpCode::Exec((12, 14));
        assert!(input.serialized_size() == 18);
    }
}
