use crate::core::*;

use std::collections::HashSet;

/// This Heap represents the interpreters Heap (as in memory area, not as in data structure).
///
/// It is not any form of tree. The idea is, that I want to be able to insert an element into it,
/// to reference it, and to delete it. I don't want to care about it's key.
///
/// It uses 2 Vecs. If something is inserted, it is pushed to the vec. If it is removed,
/// the index will be stored in the 2nd Vec, and reused on the next insert. Removal anywhere is O(1),
/// which is not the case with vec.
/// The downside is, that you cannot shrink it. In case this becomes a necessity, it would be possible,
/// to additionally use a HashMap to map old indices to new ones.
/// As this is used in the interpreter and should be as fast as possible, I use unsafe functions where it
/// safes performance

#[derive(Debug)]
pub struct Heap<T> {
    data: Vec<T>,
    free_indices: Vec<usize>,
}

impl<T> Heap<T> {
    pub fn new() -> Self {
        Self {
            data: vec![],
            free_indices: vec![],
        }
    }

    pub fn push(&mut self, val: T) -> usize {
        if let Some(idx) = self.free_indices.pop() {
            self.data[idx] = val;
            idx
        } else {
            self.data.push(val);
            self.data.len() - 1
        }
    }

    pub fn delete(&mut self, idx: usize) {
        self.free_indices.push(idx);
    }

    pub unsafe fn get(&self, i: usize) -> &T {
        self.data.get_unchecked(i)
    }
}
