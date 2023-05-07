use crate::core::*;
use crate::vm;
use im::HashMap;
use std::result::Result as StdResult;
use thiserror::Error;

/// maps a symbol name to an ast-id, which can be used to look up the type
pub type Environment = Scopes<String, EnvironmentIdentifier>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvironmentIdentifier {
    AstId(usize),
    BuiltIn(usize),
}

impl From<EnvironmentIdentifier> for usize {
    fn from(value: EnvironmentIdentifier) -> Self {
        match value {
            EnvironmentIdentifier::AstId(id) => id,
            EnvironmentIdentifier::BuiltIn(id) => id,
        }
    }
}

/// The type information for each expr will be associated via it's id
pub type TypeIndex = HashMap<EnvironmentIdentifier, DataType>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not Callable")]
    NotCallable(usize),

    #[error("Undefined Symbol {1}")]
    UndefinedSymbol(usize, String),
}

impl Error {
    pub fn get_ast_id(&self) -> usize {
        match self {
            Self::NotCallable(id) => *id,
            Self::UndefinedSymbol(id, ..) => *id,
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

pub fn infer_ast_types(
    ast: &Block,
    mut env: Environment,
    mut type_idx: TypeIndex,
) -> Result<(Environment, TypeIndex)> {
    match ast {
        Block(_, phrases) => {
            for phrase in phrases {
                (env, type_idx) = infer_phrase_type(phrase, env, type_idx)?;
            }
            Ok((env, type_idx))
        }
    }
}

pub fn infer_phrase_type(
    phrase: &Phrase,
    env: Environment,
    type_idx: TypeIndex,
) -> Result<(Environment, TypeIndex)> {
    let Phrase::Expr(_, expr) = phrase;
    infer_expr_type(expr, env, type_idx)
}

pub fn infer_expr_type(
    expr: &Expr,
    mut env: Environment,
    mut type_idx: TypeIndex,
) -> Result<(Environment, TypeIndex)> {
    use Expr::*;
    match expr {
        Call { id, callee, args } => {
            (_, type_idx) = infer_expr_type(callee, env.clone(), type_idx)?;
            for arg in args {
                (_, type_idx) = infer_expr_type(arg, env.clone(), type_idx)?;
            }
            let callee_id = callee.get_id();
            let callee_type = type_idx.get(&callee_id).unwrap();
            let expr_type = callee_type
                .try_get_return_type()
                .ok_or_else(|| Error::NotCallable(*id))?;
            type_idx.insert(EnvironmentIdentifier::AstId(*id), expr_type);
            Ok((env, type_idx))
        }
        IntLit(id, _) => {
            type_idx.insert(EnvironmentIdentifier::AstId(*id), DataType::int());
            Ok((env, type_idx))
        }
        StrLit(id, _) => {
            type_idx.insert(EnvironmentIdentifier::AstId(*id), DataType::int());
            Ok((env, type_idx))
        }
        Let {
            id,
            symbol_name,
            value_expr,
        } => {
            (_, type_idx) = infer_expr_type(value_expr, env.clone(), type_idx)?;
            let rhs_type = type_idx.get(&value_expr.get_id()).unwrap();
            let id = EnvironmentIdentifier::AstId(*id);
            type_idx.insert(id, rhs_type.clone());
            env.add_entry(symbol_name.to_owned(), id);
            Ok((env, type_idx))
        }
        Symbol(id, name) => {
            let symbol_def_id = env
                .find_entry(name)
                .ok_or_else(|| Error::UndefinedSymbol(*id, name.to_owned()))?;
            let t = type_idx.get(symbol_def_id).unwrap();
            type_idx.insert(EnvironmentIdentifier::AstId(*id), t.clone());
            Ok((env, type_idx))
        }
    }
}

/// generates and environment and a type index that already contains the built ins
pub fn inference_start() -> (Environment, TypeIndex) {
    let mut env = Scopes::default();
    let mut type_idx = TypeIndex::default();
    for (i, bi) in vm::BUILT_INS.iter().enumerate() {
        let id = EnvironmentIdentifier::BuiltIn(i);
        env.add_entry(bi.to_string(), id);
        type_idx.insert(
            id,
            DataType::Callable(
                CallableType::Builtin,
                Box::new(
                    vm::signatures(bi).expect(&format!("No signature found for built in: {}", bi)),
                ),
            ),
        );
    }
    (env, type_idx)
}
