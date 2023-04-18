use crate::{ast::Block, opcode::OpCode, utils};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default)]
pub struct Intermediate {
    /// Basically the program
    pub text: Vec<OpCode>,
    /// Data section
    pub data: Vec<Data>,
    /// AST node from which the corresponding OpCode was generated
    pub ast_ids: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Data {
    String(String),
    Vec(Vec<Data>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinalHeader {
    pub version: [u16; 3],
    pub ast: Block,
    pub ast_ids: Vec<usize>,
    /// contains the mapping from offset in the byte-Vec to
    /// index of opcode in OpCode-Vec
    pub index: HashMap<usize, usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Final {
    pub text: Vec<u8>,
    pub data: Vec<Data>,
    pub header: FinalHeader,
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

impl Data {
    pub fn get_as<'a, T: DataAs<'a>>(&'a self) -> Option<T> {
        <T as DataAs>::get_as(&self)
    }
}

pub trait DataAs<'a>: std::fmt::Debug + Sized {
    fn get_as(data: &'a Data) -> Option<Self>;
}

impl<'a> DataAs<'a> for &'a str {
    fn get_as(data: &'a Data) -> Option<&'a str> {
        if let Data::String(s) = data {
            Some(&s)
        } else {
            None
        }
    }
}

impl<'a, T: DataAs<'a>> DataAs<'a> for Vec<T> {
    fn get_as(data: &'a Data) -> Option<Vec<T>> {
        if let Data::Vec(vec) = data {
            utils::sequence_option(vec.iter().map(|d| d.get_as()))
        } else {
            None
        }
    }
}
