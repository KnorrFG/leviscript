//! handles the steps ast -> intermediate byte code -> final bytecode
//!
//! The compilation from ast to intermediate bytecode is handled via the Compilable trait,
//! that is implemented by all Ast node types. The definition and implementation of this trait
//! reside in this file.

use std::io::Write;
use thiserror::Error;

use crate::core::*;
use crate::utils;
use std::result::Result as StdResult;

/// The errors that can occur during compilation
#[derive(Error, Debug)]
pub enum CompilationError {
    #[error("Undefined Symbol: {name}")]
    UndefinedSymbol { ast_id: usize, name: String },

    #[error("A compiler bug was detected: {msg}")]
    CompilerBug { ast_id: usize, msg: String },
}

type Result<T> = StdResult<T, CompilationError>;

/// Handles first part of compilation
pub trait Compilable {
    fn compile(&self, scopes: Scopes, stack_info: StackInfo) -> Result<ImByteCode>;
}

/// return a compilation error immediately
macro_rules! compilation_error {
    ($($err:tt)+) => {
        return Err(CompilationError::$($err)*);
    };
}

/// return a compiler error immediately
macro_rules! compiler_bug {
    ($ast_id: expr, $msg:literal $(, $args: expr)*) => {
        return Err(CompilationError::CompilerBug{
            ast_id: $ast_id,
            msg: format!($msg $(, $args)*)});
    };
}

/// used to implement the Comilable trait for the different ast-noded, saves some boilerplate
macro_rules! impl_compilable {
    ($t:ty: $self:ident, $scopes:ident, $stack_info:ident => $code:tt) => {
        impl Compilable for $t {
            fn compile(&$self, $scopes: Scopes, $stack_info: StackInfo) -> Result<ImByteCode> {
                $code
            }
        }
    };
}

impl Compilable for Expr {
    fn compile(&self, mut scopes: Scopes, mut stack_info: StackInfo) -> Result<ImByteCode> {
        let mut data = vec![];
        let mut text = vec![];
        let mut ast_ids = vec![];

        use Expr::*;
        Ok(match self {
            XExpr { exe, args, id } => {
                // handle the first argument
                fn compile_atom(
                    data: &mut Vec<Value<()>>,
                    text: &mut Vec<OpCode>,
                    atom: &XExprAtom,
                    scopes: &mut Scopes,
                ) -> Result<()> {
                    match atom {
                        XExprAtom::Str(_, val) => {
                            // if it's a string, put it into the data section, and put an instruction
                            // into the opcode to push a reference to the data section onto the stack
                            data.push(val.to_owned().into());
                            text.push(OpCode::PushDataSecRef(data.len() - 1));
                        }
                        XExprAtom::Ref(id, name) => {
                            // if it's a reference, the data already lives somewhere, and we simply put
                            // the reference to that somewhere on the stack
                            let SymbolInfo {
                                stack_idx,
                                dtype: _,
                            } = get_symbol_or_raise(&scopes, name, *id)?;
                            text.push(OpCode::RepushStackEntry(*stack_idx));
                        }
                    }
                    Ok(())
                }

                compile_atom(&mut data, &mut text, &exe, &mut scopes)?;
                // the second argument is a variadic vec of Data. That means we put all the
                // args on the stack, and then an entry saying how many that are
                let n = args.len();
                for arg in args {
                    compile_atom(&mut data, &mut text, &arg, &mut scopes)?;
                }

                // stack info wasn't updated, because it would be removed again anyway
                text.push(OpCode::PushPrimitive(n.into()));
                text.push(OpCode::Exec);

                // no need to update stack info. the exec opcode will pop all the args we
                // just put on it
                ImByteCode {
                    text,
                    data,
                    ast_ids,
                    stack_info,
                    scopes,
                }
            }
            Let {
                id,
                symbol_name,
                value_expr,
            } => {
                // if the child expression is a symbol, the scopes are updated.
                // the code in the else block would work, but creates unnessessary
                // indirection
                if let Expr::Symbol(id, name) = value_expr.as_ref() {
                    let symbol_info = get_symbol_or_raise(&scopes, name, *id)?;
                    scopes.add_symbol_info(symbol_name.to_owned(), symbol_info.clone());
                    ImByteCode {
                        text,
                        data,
                        ast_ids,
                        stack_info,
                        scopes,
                    }
                } else {
                    let mut value_code = value_expr.compile(scopes.clone(), stack_info.clone())?;
                    std::io::stdout().flush();
                    if !(stack_info.len() + 1 == value_code.stack_info.len()) {
                        compiler_bug!(
                            *id,
                            "During compilation of let statement. \n\
                            The stack size should have increase by one through the code\n\
                            representing the rhs of the let-binding. But it didn't.\n\
                            rhs-code:\n{:#?}\n\nrhs-expr:\n{:#?}",
                            value_code,
                            value_expr
                        );
                    }
                    let info = value_code.stack_info.back().unwrap();
                    value_code.scopes.add_symbol(
                        symbol_name.clone(),
                        value_code.stack_info.len() - 1,
                        info.dtype.clone(),
                    );
                    value_code
                }
            }
            StrLit(id, elems) => {
                let mut current_byte_code = ImByteCode {
                    text,
                    data,
                    ast_ids,
                    stack_info,
                    scopes,
                };
                use StrLitElem::*;
                let n = elems.len();
                for e in elems {
                    match &e {
                        PureStrLit(_, val) => {
                            current_byte_code.data.push(Value::Str(val.clone()));
                            current_byte_code
                                .text
                                .push(OpCode::PushDataSecRef(current_byte_code.data.len() - 1));
                        }
                        Symbol(symbol_ast_id, name) => {
                            let info = get_symbol_or_raise(
                                &current_byte_code.scopes,
                                name,
                                *symbol_ast_id,
                            )?;
                            current_byte_code
                                .text
                                .push(OpCode::RepushStackEntry(info.stack_idx));
                        }
                        Expr(_, sub_expr) => {
                            let sub_expr_code = sub_expr.compile(
                                current_byte_code.scopes.clone(),
                                current_byte_code.stack_info.clone(),
                            )?;
                            current_byte_code.append(sub_expr_code);
                            // the sub expression will have updated the stack info.
                            // We dont want this, this is transparent to the outside world.
                            // because after this operation the stack is one bigger than before
                            // and what happens in between doesn't matter. So this must be
                            // deleted. This might make problems, because the subexpression
                            // will have wrong information about the stack size, so it might push
                            // invalid heap refs. So i should probably always update the stack
                            // info when i manip the stack.
                            // but then that should go hand in hand with pushing the instruction
                            current_byte_code.stack_info.pop_back();
                        }
                    }
                }
                current_byte_code.text.push(OpCode::PushPrimitive(n.into()));
                current_byte_code.text.push(OpCode::StrCat);
                // there will the value and the ref to that value on the Stack
                // StackInfo should consist of a wrapper around DataType and Value
                current_byte_code.stack_info.push_back(DataInfo {
                    dtype: DataType::Str,
                    ast_id: *id,
                });
                current_byte_code
            }
            Symbol(ast_id, name) => {
                let symbol_info = get_symbol_or_raise(&scopes, name, *ast_id)?;
                text.push(OpCode::RepushStackEntry(symbol_info.stack_idx));
                let mut stack_info = stack_info.clone();
                stack_info.push_back(DataInfo {
                    dtype: DataType::Ref(Box::new(symbol_info.dtype.clone())),
                    ast_id: *ast_id,
                });
                ImByteCode {
                    text,
                    data,
                    ast_ids,
                    stack_info,
                    scopes: scopes.clone(),
                }
            }
            IntLit(id, val) => {
                text.push(OpCode::PushPrimitive(CopyValue::Int(*val)));
                let mut stack_info = stack_info.clone();
                stack_info.push_back(DataInfo {
                    dtype: DataType::Int,
                    ast_id: *id,
                });
                ImByteCode {
                    text,
                    data,
                    ast_ids,
                    stack_info,
                    scopes,
                }
            }
        })
    }
}

impl_compilable! {  Phrase: self, scopes, stack_info  => {
    use Phrase::*;
    match self {
        Expr(_, exp) => exp.compile(scopes, stack_info)
    }
}}

impl_compilable! { Block: self, scopes, stack_info => {
    let Block(_, terms) = self;
    let mut res = ImByteCode::with_scope_and_stack(scopes.clone(), stack_info.clone());

    for term in terms {
        let c = term.compile(res.scopes.clone(), res.stack_info.clone())?;
        res.append(c);
    }
    Ok(res)
}}

pub fn intermediate_to_final(mut im: ImByteCode, ast: Block) -> FinalByteCode {
    im.text.push(OpCode::Exit(0));
    let final_opcode_sizes: Vec<_> = im.text.iter().map(|oc| oc.serialized_size()).collect();
    let index = (0..final_opcode_sizes.len()).map(|i| final_opcode_sizes[0..i].iter().sum());
    let final_index = index.enumerate().map(|(a, b)| (b, a)).collect();
    let final_text = im.text.iter().flat_map(|c| c.to_bytes()).collect();

    FinalByteCode {
        text: final_text,
        data: im.data,
        header: FinalByteCodeHeader {
            version: utils::get_version(),
            ast,
            ast_ids: im.ast_ids,
            index: final_index,
        },
    }
}

fn get_symbol_or_raise<'a>(
    scopes: &'a Scopes,
    name: &str,
    ast_id: usize,
) -> Result<&'a SymbolInfo> {
    if let Some(i) = scopes.find_index_for(name) {
        Ok(i)
    } else {
        compilation_error!(UndefinedSymbol {
            ast_id,
            name: name.into()
        });
    }
}
