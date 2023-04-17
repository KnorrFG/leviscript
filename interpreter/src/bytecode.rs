use crate::opcode::OpCode;

#[derive(Default)]
pub struct Intermediate {
    /// Basically the program
    pub text: Vec<OpCode>,
    /// Data section
    pub data: Vec<Data>,
    /// AST node from which the corresponding OpCode was generated
    pub ast_ids: Vec<usize>,
}

pub enum Data {
    String(String),
    Vec(Vec<Data>),
}

impl Intermediate {
    /// does Vec::append for every member. Also patches addresses so they stay correct
    pub fn append(&mut self, other: &mut Self) {
        for code in &mut other.text {
            code.offset_data_section_addr(self.data.len());
        }
        self.text.append(&mut other.text);
        self.data.append(&mut other.data);
        self.ast_ids.append(&mut other.ast_ids);
    }
}
