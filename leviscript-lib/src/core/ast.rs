//! Contains the AST types. All anonymous structs and variants start with a usize, which is their
//! ID. The Id refers to the index in the Span-vec that is returned together with the ast

use serde::{Deserialize, Serialize};

/// represents multiple phrases that are executed one after another
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block(pub usize, pub Vec<Phrase>);

/// either an expression or a statement
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Phrase {
    Expr(usize, Expr),
}

/// represents an expression
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Expr {
    /// represents an X-Expression
    XExpr {
        exe: XExprAtom,
        args: Vec<XExprAtom>,
        id: usize,
    },
    /// represents a string literal
    StrLit(usize, Vec<StrLitElem>),
    /// represents variable defintion
    Let {
        id: usize,
        symbol_name: String,
        value_expr: Box<Expr>,
    },
    /// represents a symbol, respectivly it's value
    Symbol(usize, String),
    /// represents an integer literal
    IntLit(usize, i64),
}

/// represents part of a string literal
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StrLitElem {
    PureStrLit(usize, String),
    Symbol(usize, String),
    Expr(usize, Expr),
}

/// represents an argment to an X-Expression
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum XExprAtom {
    Ref(usize, String),
    Str(usize, String),
}
