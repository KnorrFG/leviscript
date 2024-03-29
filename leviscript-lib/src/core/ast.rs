//! Contains the AST types. All anonymous structs and variants start with a usize, which is their
//! ID. The Id refers to the index in the Span-vec that is returned together with the ast

use std::iter;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::core::ast_macros::{define_ast_node_ref, mk_enum_node};
// this is needed because the mk_enum_node macro generates an implementation for it
use crate::type_inference::EnvironmentIdentifier;

use proc_macros::AstNode;

/// represents stuff that works with all AST-Nodes
pub trait AstNode {
    fn get_id(&self) -> EnvironmentIdentifier;
    fn children(&self) -> Box<dyn Iterator<Item = AstNodeRef> + '_>;
    fn get_node_ref(&self) -> AstNodeRef;
    fn iter(&self) -> Box<dyn Iterator<Item = AstNodeRef> + '_> {
        Box::new(iter::once(self.get_node_ref()).chain(self.children().flat_map(|c| c.iter())))
    }
}

/// represents multiple phrases that are executed one after another
#[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
pub struct Block(pub usize, #[children] pub Vec<Phrase>);

mk_enum_node! { Phrase, Expr }

#[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
pub struct StrLit(pub usize, pub String);

#[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
pub struct Symbol(pub usize, pub String);

//#[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
//pub struct Keyword(pub usize, pub String);

#[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
pub struct IntLit(pub usize, pub i64);

#[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
pub struct Let {
    pub id: usize,
    pub symbol_name: String,

    #[child]
    pub value_expr: Box<Expr>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
pub struct Call {
    pub id: usize,

    #[child]
    pub callee: Box<Expr>,

    #[children]
    pub args: Vec<Expr>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
pub struct FnFragment {
    pub id: usize,
    #[children]
    pub args: Vec<ArgDef>,

    #[child]
    pub body: Box<Expr>,
}

#[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
pub struct ArgDef {
    pub id: usize,
    pub name: String,
}

mk_enum_node! { Expr, StrLit, Symbol, IntLit, Let, Call, FnFragment, Block}

define_ast_node_ref! {
    Block, Phrase, StrLit, Symbol, IntLit, Let, Call, FnFragment, Expr, ArgDef,
}

impl<'a> AstNodeRef<'a> {
    pub fn walk<F, E>(self, f: &mut F) -> Result<(), E>
    where
        F: FnMut(AstNodeRef) -> Result<(), E>,
    {
        f(self)?;
        for child in self.children() {
            child.walk(f)?;
        }
        Ok(())
    }
}
