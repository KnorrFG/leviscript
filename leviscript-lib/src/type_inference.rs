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

    #[error("For now, unused variables are forbidden: {1}")]
    UnusedVar(usize, String),

    #[error("Is not Callable")]
    CallingNonCallable(usize),
}

impl Error {
    pub fn get_ast_id(&self) -> usize {
        match self {
            Self::NotCallable(id) => *id,
            Self::CallingNonCallable(id) => *id,
            Self::UndefinedSymbol(id, ..) => *id,
            Self::UnusedVar(id, ..) => *id,
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
        mut env: Environment,
        mut type_idx: TypeIndex,
    ) -> Result<(Environment, TypeIndex)> {
        let mut known_types = vec![None; self.args.len()];
        // iter body and look for calls that give away the type of our arguments
        for node in self.body.iter() {
            if let AstNodeRef::Call(Call {
                callee,
                args: call_args,
                ..
            }) = node
            {
                let callee_id = match callee.as_ref() {
                    Expr::Symbol(Symbol(_, callee_name)) => {
                        env.find_entry(callee_name).expect("can't find callee type")
                    }
                    _ => panic!("not supported yet"),
                };
                let indices = find_matching_indices(&self.args, call_args);
                let DataType::Callable(_, callee_sign) = type_idx.get(&callee_id).unwrap()
                else { return Err(Error::CallingNonCallable((*callee_id).into())) };
                for (a_i, ca_i) in indices {
                    let ca_t = callee_sign.get_nth_arg(ca_i).unwrap();
                    if let Some(t) = ca_t.concrete_type() {
                        known_types[a_i] = Some(t.clone());
                    } else {
                        panic!("For now, this doesn't work");
                    }
                }
            }
        }

        // insert argument types into env and type index, which is needed when infering the body
        // type
        let mut env_for_body = env.clone();
        let mut ti_for_body = type_idx.clone();
        for (arg_def, t) in self.args.iter().zip(known_types) {
            let actual_type = if let Some(t) = t {
                t.clone()
            } else {
                return Err(Error::UnusedVar(
                    arg_def.get_id().into(),
                    arg_def.name.clone(),
                ));
            };
            ti_for_body.insert(arg_def.get_id(), actual_type);
            env_for_body.add_entry(arg_def.name.clone(), arg_def.get_id());
        }

        // find out the result type
        (_, ti_for_body) = self.body.infer_types(env_for_body, ti_for_body)?;
        let res_type = ti_for_body.get(&self.body.get_id()).unwrap();

        // put own type into type_idx
        let arg_type_vec = self
            .args
            .iter()
            .map(|a| ti_for_body.get(&a.get_id()).unwrap().clone().into())
            .collect();

        let my_type = DataType::Callable(
            CallableType::FnFragment,
            Box::new(
                Signature::new()
                    .args(arg_type_vec)
                    .result(res_type.clone().into()),
            ),
        );
        type_idx.insert(self.get_id(), my_type);
        Ok((env, type_idx))
    }
}

/// for each symbol in the call args that is one of the arguments, it returns a mapping from
/// arg_name index to call_args index
fn find_matching_indices(arg_names: &[ArgDef], call_args: &[Expr]) -> Vec<(usize, usize)> {
    let mut res = vec![];
    for (ca_i, ca) in call_args.iter().enumerate() {
        if let Expr::Symbol(Symbol(_, call_arg_name)) = ca {
            for (an_i, an) in arg_names.iter().enumerate() {
                if call_arg_name == &an.name {
                    res.push((an_i, ca_i));
                }
            }
        }
    }
    res
}

#[cfg(test)]
#[test]
fn test_find_matching_indices() {
    let searched_names = [
        ArgDef {
            id: 0,
            name: "foo".into(),
        },
        ArgDef {
            id: 0,
            name: "bar".into(),
        },
    ];
    let search_space = [Expr::IntLit(IntLit(0, 1))];
    let res = find_matching_indices(&searched_names, &search_space);
    assert_eq!(Vec::<(usize, usize)>::new(), res);

    let search_space = [
        Expr::IntLit(IntLit(0, 1)),
        Expr::Symbol(Symbol(0, "foo".into())),
    ];
    let res = find_matching_indices(&searched_names, &search_space);
    assert_eq!(vec![(0, 1)], res);
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
