//! Contains the AST types. All anonymous structs and variants start with a usize, which is their
//! ID. The Id refers to the index in the Span-vec that is returned together with the ast

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block(pub usize, pub Vec<Term>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Term {
    Expr(usize, Expr),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Expr {
    XExpr {
        exe: XExprAtom,
        args: Vec<XExprAtom>,
        id: usize,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum XExprAtom {
    Ref(usize, String),
    Str(usize, String),
}
