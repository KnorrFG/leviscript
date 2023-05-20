//! Contains the AST types. All anonymous structs and variants start with a usize, which is their
//! ID. The Id refers to the index in the Span-vec that is returned together with the ast

use serde::{Deserialize, Serialize};

use crate::type_inference::EnvironmentIdentifier;

/// represents stuff that works with all AST-Nodes
pub trait AstNode {
    fn get_id(&self) -> EnvironmentIdentifier;
}

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
    // XExpr {
    //     exe: XExprAtom,
    //     args: Vec<XExprAtom>,
    //     id: usize,
    // },
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
    /// represents a call to a (builtin) function
    Call {
        id: usize,
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Block(usize, Box<Block>),
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

impl AstNode for Block {
    fn get_id(&self) -> EnvironmentIdentifier {
        EnvironmentIdentifier::AstId(self.0)
    }
}

impl AstNode for Phrase {
    fn get_id(&self) -> EnvironmentIdentifier {
        EnvironmentIdentifier::AstId(match self {
            Self::Expr(id, _) => *id,
        })
    }
}

impl AstNode for Expr {
    fn get_id(&self) -> EnvironmentIdentifier {
        use Expr::*;
        EnvironmentIdentifier::AstId(match self {
            // XExpr { id, .. } => id,
            Block(id, _) => *id,
            StrLit(id, ..) => *id,
            Let { id, .. } => *id,
            Symbol(id, ..) => *id,
            IntLit(id, ..) => *id,
            Call { id, .. } => *id,
        })
    }
}

impl AstNode for StrLitElem {
    fn get_id(&self) -> EnvironmentIdentifier {
        EnvironmentIdentifier::AstId(match self {
            Self::PureStrLit(id, ..) => *id,
            Self::Symbol(id, ..) => *id,
            Self::Expr(id, ..) => *id,
        })
    }
}

impl AstNode for XExprAtom {
    fn get_id(&self) -> EnvironmentIdentifier {
        EnvironmentIdentifier::AstId(match self {
            Self::Ref(id, ..) => *id,
            Self::Str(id, ..) => *id,
        })
    }
}
