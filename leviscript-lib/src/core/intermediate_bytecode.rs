use crate::core::*;

/// represents byte code in a more abstract form
#[derive(Debug, Default, Clone)]
pub struct ImByteCode {
    /// Basically the program
    pub text: Vec<OpCode>,
    /// Data section
    pub data: Vec<ComptimeValue>,
    /// AST node from which the corresponding OpCode was generated
    pub ast_ids: Vec<usize>,
    /// the size that the stack will have, after this code was executed
    pub stack_info: StackInfo,
    /// Represents the available symbols
    pub scopes: Scopes,
}

/// represents an int index into the data-section vector
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DataSecRef(pub usize);

impl ImByteCode {
    /// does Vec::append for every member. Also patches addresses so they stay correct
    /// stack_info and scopes from other are used unchanged
    pub fn append(&mut self, other: Self) {
        let Self {
            mut text,
            mut data,
            mut ast_ids,
            stack_info,
            scopes,
        } = other;
        for code in &mut text {
            code.offset_data_section_addr(self.data.len());
        }
        self.text.append(&mut text);
        self.data.append(&mut data);
        self.ast_ids.append(&mut ast_ids);
        self.stack_info = stack_info;
        self.scopes = scopes;
    }

    /// returns index to the top most stack elem
    pub fn stack_top_idx(&self) -> usize {
        self.stack_info.len() - 1
    }

    pub fn with_scope_and_stack(scopes: Scopes, stack_info: StackInfo) -> ImByteCode {
        ImByteCode {
            scopes,
            stack_info,
            ..ImByteCode::default()
        }
    }
}
