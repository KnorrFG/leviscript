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

pub trait TypeInferable {
    fn infer_types(
        &self,
        env: Environment,
        type_idx: TypeIndex,
    ) -> Result<(Environment, TypeIndex)>;
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

impl TypeInferable for Block {
    fn infer_types(
        &self,
        mut env: Environment,
        mut type_idx: TypeIndex,
    ) -> Result<(Environment, TypeIndex)> {
        let Block(id, phrases) = self;
        for phrase in phrases {
            (env, type_idx) = phrase.infer_types(env, type_idx)?;
        }
        let id = EnvironmentIdentifier::AstId(*id);
        if let Some(last_phrase) = phrases.last() {
            copy_type_info(&mut type_idx, &last_phrase.get_id(), id);
        } else {
            type_idx.insert(id, DataType::unit());
        }
        Ok((env, type_idx))
    }
}

impl TypeInferable for FnFragment {
    fn infer_types(
        &self,
        _env: Environment,
        _type_idx: TypeIndex,
    ) -> Result<(Environment, TypeIndex)> {
        todo!();
    }
}

impl TypeInferable for Call {
    fn infer_types(
        &self,
        env: Environment,
        mut type_idx: TypeIndex,
    ) -> Result<(Environment, TypeIndex)> {
        let Call { id, callee, args } = self;
        (_, type_idx) = callee.infer_types(env.clone(), type_idx)?;
        for arg in args {
            (_, type_idx) = arg.infer_types(env.clone(), type_idx)?;
        }
        let callee_id = callee.get_id();
        let callee_type = type_idx.get(&callee_id).unwrap();
        let fn_result = callee_type
            .try_get_return_type()
            .ok_or_else(|| Error::NotCallable(*id))?;
        let Some(expr_type) = fn_result .concrete_type() else {
            panic!("Currently not supported");
        };
        type_idx.insert(EnvironmentIdentifier::AstId(*id), expr_type.clone());
        Ok((env, type_idx))
    }
}

impl TypeInferable for IntLit {
    fn infer_types(
        &self,
        env: Environment,
        mut type_idx: TypeIndex,
    ) -> Result<(Environment, TypeIndex)> {
        type_idx.insert(EnvironmentIdentifier::AstId(self.0), DataType::int());
        Ok((env, type_idx))
    }
}

impl TypeInferable for StrLit {
    fn infer_types(
        &self,
        env: Environment,
        mut type_idx: TypeIndex,
    ) -> Result<(Environment, TypeIndex)> {
        type_idx.insert(EnvironmentIdentifier::AstId(self.0), DataType::str());
        Ok((env, type_idx))
    }
}

impl TypeInferable for Let {
    fn infer_types(
        &self,
        mut env: Environment,
        mut type_idx: TypeIndex,
    ) -> Result<(Environment, TypeIndex)> {
        let Let {
            id,
            symbol_name,
            value_expr,
        } = self;
        (_, type_idx) = value_expr.infer_types(env.clone(), type_idx)?;
        let rhs_type = type_idx.get(&value_expr.get_id()).unwrap();
        let id = EnvironmentIdentifier::AstId(*id);
        type_idx.insert(id, rhs_type.clone());
        env.add_entry(symbol_name.to_owned(), id);
        Ok((env, type_idx))
    }
}

impl TypeInferable for Symbol {
    fn infer_types(
        &self,
        env: Environment,
        mut type_idx: TypeIndex,
    ) -> Result<(Environment, TypeIndex)> {
        let Symbol(id, name) = self;
        let symbol_def_id = env
            .find_entry(name)
            .ok_or_else(|| Error::UndefinedSymbol(*id, name.to_owned()))?;
        let t = type_idx.get(symbol_def_id).unwrap();
        type_idx.insert(EnvironmentIdentifier::AstId(*id), t.clone());
        Ok((env, type_idx))
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

fn copy_type_info(
    type_idx: &mut TypeIndex,
    from: &EnvironmentIdentifier,
    to: EnvironmentIdentifier,
) {
    type_idx.insert(to, type_idx.get(from).unwrap().clone());
}
