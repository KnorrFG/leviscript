use proc_macro::TokenStream;
mod ast_node;
mod opcode;

/// Used on the OpCode enum.
///
/// An opcode is basically an id paired with
/// arguments, and enums lend them selfes naturally to this usecase, however
/// opcodes will have to exist as a byte sequence, and for speed purposes
/// the binary representation will simply be achived via raw pointer cast.
/// The problem is, that the repr of an enum is as large as the largest variant,
/// and I don't want to waste that space. So the enum will be encoded as discriminant
/// plus the repr of it's data.
///
/// This Macro generates the following:
/// * a discriminant for each variant
/// * A function get_id() to get from variant to discriminant
/// * Self::to_bytes(&self) -> `Vec<u8>`
/// * Self::serialized_size(&self) -> usize
/// * Self::serialized_size_of(u16) -> usize
/// * unsafe Self::dispatch_discriminant(u16, *const u8, vm::Memory) -> *const u8
///   Every Opcode will be executed at some point by the vm.
///   this function will call the associated function for an opcode. That fn must be called
///   `exec_<opcode in lowercase>(pc: *const u8, mem: &mut Memory)`. And the symbol must be
///   available when dispatch_discriminant is called
/// * get_body!(opcode) -> body_type macro
///   for opcodes that have arguments, it adds 2 to the pc and then casts it to a ref of the
///   argument type. Look at the `exec_<opcode in lowercase>` fns for usage examples
#[proc_macro_derive(OpCode)]
pub fn convert(tokens: TokenStream) -> TokenStream {
    opcode::opcode_impl(tokens)
}

/// Implements the `[leviscript-lib::core::AstNode]` Trait
///
/// If put on an enum, that enum must only contain variants with a single unnamed field, and that
/// field must implement ast-node.
/// If put on a struct with named fields, the struct must contain a field id:usize, and if put on a
/// struct with unnamed fields, the first field must be a usize and will be used as id field.
/// fields that hold child nodes must be anotated with `#[child]` if they have a single element, or
/// `#[children]` if they implement Iterator over child nodes. That nodes must implement AstNode
/// and Into<AstNodeRef>
#[proc_macro_derive(AstNode, attributes(child, children))]
pub fn derive_ast_note(tokens: TokenStream) -> TokenStream {
    ast_node::ast_node_impl(tokens)
}
