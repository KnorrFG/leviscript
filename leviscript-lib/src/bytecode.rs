use crate::{
    ast::Block,
    opcode::{DataRef, OpCode},
};
use im::hashmap::HashMap as ImHashMap;
use im::vector::Vector as ImVec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Scope = ImHashMap<String, usize>;
pub type StackInfo = ImVec<DataInfo>;

#[derive(Debug, Clone)]
pub struct DataInfo {
    pub dtype: DataType,
    pub ast_id: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Intermediate {
    /// Basically the program
    pub text: Vec<OpCode>,
    /// Data section
    pub data: Vec<Data>,
    /// AST node from which the corresponding OpCode was generated
    pub ast_ids: Vec<usize>,
    /// the size that the stack will have, after this code was executed
    pub stack_info: StackInfo,
    /// Represents the available symbols
    pub scopes: Scopes,
}

#[derive(Debug, Clone)]
pub struct Scopes {
    /// Each Entry in the vec is a new scope, the last is the inner most one.
    /// In a scope, there is a mapping from symbol name to stack_index at which the
    /// corresponding variable can be found. The first scope is assumend to be the
    /// global scope
    pub scopes: ImVec<Scope>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Data {
    String(String),
    Vec(Vec<Data>),
    Ref(DataRef),
}

#[derive(Debug, Clone)]
pub enum DataType {
    String,
    Vec(Box<DataType>),
    Ref(DataRef),
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
    /// stack_info and scopes from other are used unchanged
    pub fn append(&mut self, mut other: Self) {
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
        for d in &mut data {
            d.offset_data_section_addr(self.data.len());
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

    pub fn with_scope_and_stack(scopes: Scopes, stack_info: StackInfo) -> Intermediate {
        Intermediate {
            scopes,
            stack_info,
            ..Intermediate::default()
        }
    }
}

impl Data {
    pub fn get_as<'a, T: DataAs<'a>>(&'a self) -> Option<T> {
        <T as DataAs>::get_as(&self)
    }

    pub fn offset_data_section_addr(&mut self, offset: usize) {
        use Data::*;
        match self {
            String(_) => {}
            Vec(ds) => {
                for d in ds {
                    d.offset_data_section_addr(offset);
                }
            }
            Ref(r) => {
                if let DataRef::DataSectionIdx(i) = r {
                    *i += offset;
                }
            }
        }
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

impl Default for Scopes {
    fn default() -> Self {
        Scopes {
            scopes: ImVec::unit(ImHashMap::new()),
        }
    }
}

impl Scopes {
    pub fn open_new(&mut self) {
        self.scopes.push_back(ImHashMap::new());
    }

    pub fn collapse_innermost(&mut self) -> Scope {
        assert!(
            self.scopes.len() > 1,
            "Tried to collapse a global scope. This is a bug"
        );
        self.scopes.pop_back().unwrap()
    }

    pub fn add_symbol(&mut self, symbol_name: String, stack_index: usize) {
        self.scopes
            .back_mut()
            .unwrap()
            .insert(symbol_name, stack_index);
    }

    pub fn find_index_for(&self, symbol_name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(idx) = scope.get(symbol_name) {
                return Some(*idx);
            }
        }
        None
    }
}

impl Final {
    pub fn pc_to_index(&self, pc: *const u8) -> usize {
        let byte_offset = pc as usize - self.text.as_ptr() as usize;
        self.header.index[&byte_offset]
    }

    pub fn pc_to_ast_id(&self, pc: *const u8) -> usize {
        self.header.ast_ids[self.pc_to_index(pc)]
    }
}
