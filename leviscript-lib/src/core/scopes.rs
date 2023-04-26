//! contains code for scope checking during compilation

use im::HashMap as ImHashMap;
use im::Vector as ImVec;

use crate::core::*;

/// A scope is a Hashmap from symbol name to symbol info,
/// it uses an imutable hashmap to have structural sharing
pub type Scope = ImHashMap<String, SymbolInfo>;

/// represents the scope hirarchy. Used during compilation
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
    /// open a new scope
    pub fn open_new(&mut self) {
        self.scopes.push_back(ImHashMap::new());
    }

    /// collapse the innermost scope
    pub fn collapse_innermost(&mut self) -> Scope {
        assert!(
            self.scopes.len() > 1,
            "Tried to collapse a global scope. This is a bug"
        );
        self.scopes.pop_back().unwrap()
    }

    /// add a symbol to the innermost scope
    pub fn add_symbol(&mut self, symbol_name: String, stack_idx: usize, dtype: DataType) {
        self.scopes
            .back_mut()
            .unwrap()
            .insert(symbol_name, SymbolInfo { stack_idx, dtype });
    }

    /// add a symbol to the innermost scope
    pub fn add_symbol_info(&mut self, symbol_name: String, info: SymbolInfo) {
        self.scopes.back_mut().unwrap().insert(symbol_name, info);
    }

    /// returns the information about a symbol if it can be found.
    ///
    /// starts searching in the innermost scope, and goes outwards,
    /// if the symbol is not in the scope. Returns None if the symbol is not
    /// in any scope
    pub fn find_index_for(&self, symbol_name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(symbol_name) {
                return Some(info);
            }
        }
        None
    }
}
