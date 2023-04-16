use proc_macros::ByteConvertible;

/// Representing Opcodes, all variants must have zero or one member.
/// This way, it's possible to simply convert between an object and it's raw representation
#[derive(Debug, Clone, Copy, ByteConvertible)]
pub enum Opcode {
    /// The first usize is the index of the stack at which the executable name
    /// is stored, and the second usize is the index of the stack at which
    /// The vec with the arguments is stored
    Exec((usize, usize)),
}
