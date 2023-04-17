use thiserror::Error;

use crate::ast::*;
use crate::bytecode::{self, Data};
use crate::opcode::{DataRef, OpCode};
use crate::utils;

pub trait Compilable {
    fn compile(&self) -> CompilationResult;
}

pub type CompilationResult = Result<bytecode::Intermediate, CompilationError>;

#[derive(Error, Debug)]
pub enum CompilationError {}

macro_rules! impl_compilable {
    ($self:ident, $t:ty => $code:tt) => {
        impl Compilable for $t {
            fn compile(&$self) -> CompilationResult {
                $code
            }
        }
    };
}

impl_compilable! { self, Expr => {
   let mut data = vec![];
   let mut text = vec![];
   let mut ast_ids = vec![];

   use Expr::*;
   match self {
       XExpr { exe, args, id } => {
            let XExprAtom::Str(_, val) = exe else { unimplemented!(); };
            let exe_idx = data.len();
            data.push(Data::String(val.to_owned()));
            let args: Vec<_> = args.iter().map(|atom| {
                let  XExprAtom::Str(_, val) = atom else { unimplemented!(); };
                Data::String(val.to_owned())}).collect();
            let arg_idx = data.len();
            data.push(Data::Vec(args));
            text.push(OpCode::Exec((DataRef::DataSectionIdx(exe_idx), DataRef::DataSectionIdx(arg_idx))));
            ast_ids.push(*id)
       }
   };
   Ok(bytecode::Intermediate { text, data, ast_ids})
}}

impl_compilable! { self, Term => {
    use Term::*;
    match self {
        Expr(_, exp) => exp.compile()
    }
}}

impl_compilable! { self, Block => {
    let Block(_, terms) = self;
    let compiled_terms = utils::sequence_result(terms.iter().map(|t| t.compile()))?;

    let mut res = bytecode::Intermediate::default();
    for mut comp_result in compiled_terms {
        res.append(&mut comp_result);
    }
    Ok(res)
}}

pub fn intermediate_to_final(mut im: bytecode::Intermediate, ast: Block) -> bytecode::Final {
    im.text.push(OpCode::Exit(0));
    let final_opcode_sizes: Vec<_> = im.text.iter().map(|oc| oc.serialized_size()).collect();
    let index = (0..final_opcode_sizes.len()).map(|i| final_opcode_sizes[0..i].iter().sum());
    let final_index = index.enumerate().map(|(a, b)| (b, a)).collect();
    let final_text = im.text.iter().flat_map(|c| c.to_bytes()).collect();

    bytecode::Final {
        text: final_text,
        data: im.data,
        header: bytecode::FinalHeader {
            version: utils::get_version(),
            ast,
            ast_ids: im.ast_ids,
            index: final_index,
        },
    }
}
