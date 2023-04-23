use thiserror::Error;

use crate::ast::*;
use crate::bytecode::{self, Data, DataInfo, DataType, Scopes, StackInfo};
use crate::opcode::{DataRef, OpCode};
use crate::utils;

pub trait Compilable {
    fn compile(&self, scopes: &Scopes, stack_info: &StackInfo) -> CompilationResult;
}

pub type CompilationResult = Result<bytecode::Intermediate, CompilationError>;

#[derive(Error, Debug)]
pub enum CompilationError {
    #[error("Undefined Symbol: {name}")]
    UndefinedSymbol { ast_id: usize, name: String },

    #[error("A compiler bug was detected: {msg}")]
    CompilerBug { ast_id: usize, msg: String },
}

macro_rules! compilation_error {
    ($($err:tt)+) => {
        return Err(CompilationError::$($err)*);
    };
}

macro_rules! compiler_bug {
    ($ast_id: expr, $msg:literal $(, $args: expr)*) => {
        return Err(CompilationError::CompilerBug{
            ast_id: $ast_id,
            msg: format!($msg $(, $args)*)});
    };
}

macro_rules! impl_compilable {
    ($t:ty: $self:ident, $scopes:ident, $stack_info:ident => $code:tt) => {
        impl Compilable for $t {
            fn compile(&$self, $scopes: &Scopes, $stack_info: &StackInfo) -> CompilationResult {
                $code
            }
        }
    };
}

impl_compilable! { Expr: self, scopes, stack_info => {
    let mut data = vec![];
    let mut text = vec![];
    let mut ast_ids = vec![];

    use Expr::*;
    Ok(match self {
        XExpr { exe, args, id } => {
            let mut atom_to_ref = |a: &XExprAtom|
                Ok(match a {
                    XExprAtom::Str(_, val) => {
                        data.push(Data::String(val.to_owned()));
                        DataRef::DataSectionIdx(data.len() - 1)
                    }
                    XExprAtom::Ref(id, name) => {
                        let idx = scopes.find_index_for(&name).ok_or_else(|| CompilationError::UndefinedSymbol { ast_id: *id, name: name.to_string() });
                        DataRef::StackIdx(idx?)
                    }
                });
            let exe_ref = atom_to_ref(exe)?;
            let args: Vec<_> = args.iter().map(|a| Ok(Data::Ref(atom_to_ref(a)?))).collect::<Result<_,_>>()?;
            let arg_idx = data.len();
            data.push(Data::Vec(args));
            text.push(OpCode::Exec((exe_ref, DataRef::DataSectionIdx(arg_idx))));
            ast_ids.push(*id);
            bytecode::Intermediate { text, data, ast_ids, stack_info: stack_info.clone(), scopes: scopes.clone()}
        }
        Let { id, symbol_name, value_expr } => {
            let mut value_code = value_expr.compile(scopes, stack_info)?;
            if !(stack_info.len() + 1 == value_code.stack_info.len()){
                compiler_bug!(*id,
                    "During compilation of let statement. \n\
                    The stack size should have increase by one through the code\n\
                    representing the rhs of the let-binding. But it didn't.\n\
                    rhs-code:\n{:?}\n\nrhs-expr:\n{:?}", value_code, value_expr);
            }
            value_code.scopes.add_symbol(symbol_name.clone(), value_code.stack_info.len() - 1);
            value_code
        }
        StrLit(id, elems) => {
            let mut strcat_args = vec![];
            let mut current_byte_code = bytecode::Intermediate {
                text, data, ast_ids, stack_info: stack_info.clone(), scopes: scopes.clone()
            };
            for elem in elems {
                use StrLitElem::*;
                match elem {
                    PureStrLit(_, val) => {
                        strcat_args.push(DataRef::DataSectionIdx(current_byte_code.data.len()));
                        current_byte_code.data.push(Data::String(val.into()));
                    }
                    Symbol(symbol_ast_id, name) => {
                        if let Some(i) = scopes.find_index_for(name) {
                            strcat_args.push(DataRef::StackIdx(i));
                        } else {
                            compilation_error!(UndefinedSymbol { ast_id: *symbol_ast_id, name: name.into()});
                        }
                    }
                    SubExpr(_, sub_expr ) => {
                        let sub_expr_code = sub_expr.compile(&current_byte_code.scopes, &current_byte_code.stack_info)?;
                        current_byte_code.append(sub_expr_code);
                        strcat_args.push(DataRef::StackIdx(current_byte_code.stack_top_idx()));
                    }
                }
            }
            current_byte_code.text.push(OpCode::StrCat(strcat_args.len()));
            for arg in strcat_args {
                current_byte_code.text.push(OpCode::DataRef(arg));
            }
            current_byte_code.stack_info.push_back(DataInfo{dtype: DataType::String, ast_id: *id});
            current_byte_code
        }
    })
}}

impl_compilable! {  Phrase: self, scopes, stack_info  => {
    use Phrase::*;
    match self {
        Expr(_, exp) => exp.compile(scopes, stack_info)
    }
}}

impl_compilable! { Block: self, scopes, stack_info => {
    let Block(_, terms) = self;
    let mut res = bytecode::Intermediate::with_scope_and_stack(scopes.clone(), stack_info.clone());

    for term in terms {
        let c = term.compile(&res.scopes, &res.stack_info)?;
        res.append(c);
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
            ast_ids: im.ast_ids.clone(),
            index: final_index,
        },
    }
}
