//! Contains the AST types. All anonymous structs and variants start with a usize, which is their
//! ID. The Id refers to the index in the Span-vec that is returned together with the ast

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block(pub usize, pub Vec<Phrase>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Phrase {
    Expr(usize, Expr),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Expr {
    XExpr {
        exe: XExprAtom,
        args: Vec<XExprAtom>,
        id: usize,
    },
    StrLit(usize, Vec<StrLitElem>),
    Let {
        id: usize,
        symbol_name: String,
        value_expr: Box<Expr>,
    },
    Symbol(usize, String),
    IntLit(usize, i64),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StrLitElem {
    PureStrLit(usize, String),
    Symbol(usize, String),
    Expr(usize, Expr),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum XExprAtom {
    Ref(usize, String),
    Str(usize, String),
}
