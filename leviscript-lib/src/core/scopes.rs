use im::HashMap as ImHashMap;
use im::Vector as ImVec;

use crate::core::*;

pub type Scope = ImHashMap<String, SymbolInfo>;

#[derive(Debug, Clone)]
pub struct Scopes {
    /// Each Entry in the vec is a new scope, the last is the inner most one.
    /// In a scope, there is a mapping from symbol name to stack_index at which the
    /// corresponding variable can be found. The first scope is assumend to be the
    /// global scope
    pub scopes: ImVec<Scope>,
}

impl Default for Scopes {
    fn default() -> Self {
        Scopes {
            scopes: ImVec::unit(ImHashMap::new()),
        }
    }
}

impl Scopes {
    pub fn open_new(&mut self) {
        self.scopes.push_back(ImHashMap::new());
    }

    pub fn collapse_innermost(&mut self) -> Scope {
        assert!(
            self.scopes.len() > 1,
            "Tried to collapse a global scope. This is a bug"
        );
        self.scopes.pop_back().unwrap()
    }

    pub fn add_symbol(&mut self, symbol_name: String, stack_idx: usize, dtype: DataType) {
        self.scopes
            .back_mut()
            .unwrap()
            .insert(symbol_name, SymbolInfo { stack_idx, dtype });
    }

    pub fn add_symbol_info(&mut self, symbol_name: String, info: SymbolInfo) {
        self.scopes.back_mut().unwrap().insert(symbol_name, info);
    }

    pub fn find_index_for(&self, symbol_name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(symbol_name) {
                return Some(info);
            }
        }
        None
    }
}
