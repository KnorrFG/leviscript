//! Scopes is a stack of Mappings. Usefull during compilation
//!
//! This is usefull at multiple moments, e.g. during type_inference, type_checking,
//! actual compilation, etc. Since you might want to create multiple instances, because
//! older instances become relevant again once a scope is closed, this uses immutable
//! datastructures, which use structural sharing

use im::HashMap as ImHashMap;
use im::Vector as ImVec;

use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::Hash;

pub type Scope<K, V> = ImHashMap<K, V>;
/// represents the scope hirarchy. Used during compilation
#[derive(Debug, Clone)]
pub struct Scopes<K, V>
where
    K: Debug + Hash + Clone + Eq,
    V: Clone + Debug,
{
    /// Each Entry in the vec is a new scope, the last is the inner most one.
    /// In a scope, there is a mapping from symbol name to stack_index at which the
    /// corresponding variable can be found. The first scope is assumend to be the
    /// global scope
    pub scopes: ImVec<Scope<K, V>>,
}

impl<K, V> Default for Scopes<K, V>
where
    K: Debug + Hash + Clone + Eq,
    V: Clone + Debug,
{
    fn default() -> Self {
        Scopes {
            scopes: ImVec::unit(ImHashMap::new()),
        }
    }
}

impl<K, V> Scopes<K, V>
where
    K: Debug + Hash + Clone + Eq,
    V: Clone + Debug,
{
    /// open a new scope
    pub fn open_new(&mut self) {
        self.scopes.push_back(ImHashMap::new());
    }

    /// collapse the innermost scope
    pub fn collapse_innermost(&mut self) -> Scope<K, V> {
        assert!(
            self.scopes.len() > 1,
            "Tried to collapse a global scope. This is a bug"
        );
        self.scopes.pop_back().unwrap()
    }

    /// add a symbol to the innermost scope
    pub fn add_entry(&mut self, key: K, val: V) {
        self.scopes.back_mut().unwrap().insert(key, val);
    }

    /// returns the information about a symbol if it can be found.
    ///
    /// starts searching in the innermost scope, and goes outwards,
    /// if the symbol is not in the scope. Returns None if the symbol is not
    /// in any scope
    pub fn find_entry<BK>(&self, key: &BK) -> Option<&V>
    where
        BK: Hash + Eq + ?Sized,
        K: Borrow<BK>,
    {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(key) {
                return Some(info);
            }
        }
        None
    }
}
