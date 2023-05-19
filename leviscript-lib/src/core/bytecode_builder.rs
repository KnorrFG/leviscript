use crate::core::*;
use im::Vector;

/// represents byte code while it's being built
///
/// It's mainly a utility tool used during compilation
/// Im not entirely sure whether there is a huge use in the immutable data structures,
/// but they are nearly as fast as their normal counterparts and it keeps the option to use them
/// Heap does currently not use immutable DS under the hood, because at runtime i want it to be as
/// fast as possible
#[derive(Debug, Clone, Default)]
pub struct ByteCodeBuilder {
    /// Basically the program
    pub text: Vector<OpCode>,
    /// Data section
    pub data: Vector<ComptimeValue>,
    /// AST node from which the corresponding OpCode was generated
    pub ast_ids: Vector<usize>,
    /// Scoped table from symbol name to stack index
    pub symbol_table: Scopes<String, usize>,
    /// Represents the state that the stack will have when the contained code is run.
    pub stack_info: Vector<DataInfo>,
    /// Represents the starting index on the stack for each scope
    pub scope_starts: Vector<usize>,
}

/// Util type for the Builder
#[derive(Debug, Clone)]
pub struct DataInfo {
    pub ast_id: usize,
    pub type_info: DataTypeInfo,
}

/// Util type for DataInfo
///
/// Some types live on the heap, and some don't I want a heap idx for those who do,
/// And I don't want one for those who don't. With an Option<usize> you can store a heap idx
/// for a stack type. This way it's not possible
#[derive(Debug, Clone)]
pub enum DataTypeInfo {
    DataSecTypeInfo { dtype: HeapType, dsec_idx: usize },
    HeapTypeInfo { dtype: HeapType, owner_idx: Owner },
    StackType(StackType),
    CallableType(CallableType, Box<Signature>),
}

#[derive(Debug, Clone)]
pub enum Owner {
    Some(usize),
    None,
    Disowned,
}

impl DataTypeInfo {
    pub fn str() -> Self {
        Self::HeapTypeInfo {
            dtype: HeapType::Str,
            owner_idx: Owner::None,
        }
    }

    pub fn int() -> Self {
        Self::StackType(StackType::Int)
    }

    pub fn into_datatype(self) -> DataType {
        match self {
            DataTypeInfo::DataSecTypeInfo { dtype, .. }
            | DataTypeInfo::HeapTypeInfo { dtype, .. } => DataType::HeapType(dtype),
            DataTypeInfo::StackType(st) => DataType::StackType(st),
            DataTypeInfo::CallableType(ct, sign) => DataType::Callable(ct, sign),
        }
    }

    pub fn sattisfies(&self, dtype: &DataType) -> bool {
        self.clone().into_datatype().sattisfies(dtype)
    }
}

impl ByteCodeBuilder {
    /// creates an entry in the symbol table for the current top value on the stack
    pub fn add_symbol_for_stack_top(&mut self, symbol_name: &str) {
        assert!(
            !self.stack_info.is_empty(),
            "tried to create symbol for stack top while stack was empty"
        );
        self.symbol_table
            .add_entry(symbol_name.into(), self.stack_info.len() - 1)
    }

    pub fn add_symbol_alias(&mut self, symbol_name: &str, alias: &str) -> Result<(), ()> {
        let info = self.symbol_table.find_entry(symbol_name).ok_or(())?;
        self.symbol_table.add_entry(alias.into(), info.clone());
        Ok(())
    }
    /// adds a value to the datasection, and pushes an Instruction to put a ref to that value onto the stack
    pub fn add_to_datasection_and_push_ref(&mut self, val: ComptimeValue, ast_id: usize) {
        self.data.push_back(val);
        let dsec_idx = self.data.len() - 1;
        self.text.push_back(OpCode::PushDataSecRef(dsec_idx));
        self.stack_info.push_back(DataInfo {
            ast_id,
            type_info: DataTypeInfo::DataSecTypeInfo {
                dtype: HeapType::Str,
                dsec_idx,
            },
        });
    }

    /// finds the stack index for the symbol and pushes an instruction to the text that coppies
    /// that entry to the stack top
    pub fn copy_symbol_target_to_stack_top(&mut self, symbol: &str) -> Result<(), ()> {
        let entry_idx = self.symbol_table.find_entry(symbol).ok_or(())?;
        self.text.push_back(OpCode::RepushStackEntry(*entry_idx));
        let mut entry = self.stack_info[*entry_idx].clone();
        if let DataTypeInfo::HeapTypeInfo {
            dtype,
            owner_idx: Owner::None,
        } = entry.type_info
        {
            entry.type_info = DataTypeInfo::HeapTypeInfo {
                dtype,
                owner_idx: Owner::Some(*entry_idx),
            }
        }
        self.stack_info.push_back(entry);
        Ok(())
    }

    /// writes an opcode to push a primitve to the stack, also updates the stack state
    pub fn push_primitive_to_stack(&mut self, val: CopyValue, ast_id: usize) {
        self.text.push_back(OpCode::PushPrimitive(val));
        let info = DataInfo {
            ast_id,
            type_info: DataTypeInfo::StackType(StackType::from(val)),
        };
        self.stack_info.push_back(info);
    }

    /// removes the n topmost entries from the stack_info.
    ///
    /// If gen_text is set to true, it will generate pop and pop_free instructions,
    /// depending on whether the stack entry is an owning reference or not.
    /// Having this option is useful, because sometimes you want those pops, e.g. for the
    /// implementation of stuff that has Scopes (e.g Expr::Call), and sometimes you dont,
    /// if you just update the stack info to account for changes that are done by OpCodes, like
    /// StrLit
    pub fn pop_stack_entries(&mut self, n: usize) {
        for _ in 0..n {
            let entry = self
                .stack_info
                .pop_back()
                .expect("stack was emtpy unexpectedly");
            let opcode = if let DataTypeInfo::HeapTypeInfo {
                owner_idx: Owner::None,
                ..
            } = entry.type_info
            {
                OpCode::PopFree
            } else {
                OpCode::Pop
            };
            self.text.push_back(opcode);
        }
    }

    /// updates stack and heap info accordingly, depending on the type
    /// use the ast_id of the expression that created the value
    pub fn create_value_in_memory(&mut self, dtype: &DataType, ast_id: usize) {
        match dtype {
            DataType::StackType(stack_type) => {
                self.stack_info.push_back(DataInfo {
                    ast_id,
                    type_info: DataTypeInfo::StackType(stack_type.clone()),
                });
            }
            DataType::HeapType(heap_type) => self.stack_info.push_back(DataInfo {
                ast_id,
                type_info: DataTypeInfo::HeapTypeInfo {
                    dtype: heap_type.clone(),
                    owner_idx: Owner::None,
                },
            }),
            DataType::Callable(_, _) => unimplemented!(),
        }
    }

    /// If the stack top is not compatible with the target type, try to fix it.
    ///
    /// If the types don't match, check whether there is a valid cast, if so,
    /// insert instructions to handle it into text.
    ///
    /// Returns whether everything is fine. A return value of false should produce
    /// a compiler error
    pub fn check_and_fix_type_of_stack_top(&mut self, target_type: &DataType) -> bool {
        let current_type = self
            .stack_info
            .last()
            .expect("attempted to access stack info, when stack was empty. This is a compiler bug")
            .type_info
            .clone()
            .into_datatype();
        if current_type.sattisfies(target_type) {
            // types already match
            true
        } else {
            // types don't match, but maybe there's a cast
            let maybe_opcode = match target_type {
                DataType::HeapType(HeapType::Str) => Some(OpCode::ToStr),
                DataType::StackType(StackType::Bool) => Some(OpCode::ToBool),
                _ => OpCode::get_cast(&current_type, target_type),
            };
            if let Some(code) = maybe_opcode {
                // There is a cast. Apply and return true.
                self.text.push_back(code);
                let old_entry = self.stack_info.pop_back().unwrap();
                self.create_value_in_memory(target_type, old_entry.ast_id);
                true
            } else {
                // There is no cast. Type error
                false
            }
        }
    }

    pub fn open_scope(&mut self) {
        self.symbol_table.open_new();
        self.scope_starts.push_back(self.stack_info.len());
    }

    pub fn collapse_scope(&mut self) {
        let res_index = self.stack_info.len() - 1;
        let scope_start_idx = self.scope_starts.back().unwrap();
        let mut res_entry = self.stack_info[res_index].clone();
        if let DataInfo {
            ast_id: _,
            type_info:
                DataTypeInfo::HeapTypeInfo {
                    dtype: _,
                    owner_idx: new_owner,
                },
        } = &mut res_entry
        {
            // The result is a heap ref. We need to check owner ship
            match new_owner {
                Owner::Some(orig_owner_idx) => {
                    if *orig_owner_idx >= *scope_start_idx {
                        // the owner will die with this scope, and we want to return the value
                        // so it can't be deleted. So we steal ownership from the owner
                        self.stack_info[*orig_owner_idx].type_info.disown();
                        *new_owner = Owner::None
                    }
                }
                Owner::None => {
                    // the ref is the owner, but it's a clone, so the original must be
                    // disowned
                    self.stack_info[res_index].type_info.disown();
                }
                Owner::Disowned => {
                    panic!("found disowned ref in stack_info");
                }
            }
        }
        self.text.push_back(OpCode::StackTopToReg(0));
        self.pop_stack_entries(self.stack_info.len() - scope_start_idx);
        self.text.push_back(OpCode::ReadReg(0));
        self.stack_info.push_back(res_entry);
        self.scope_starts.pop_back().unwrap();
        self.symbol_table.collapse_innermost();
    }

    pub fn build(mut self) -> (ByteCode, DebugInformation) {
        self.text.push_back(OpCode::Exit(0));
        let final_opcode_sizes: Vec<_> = self.text.iter().map(|oc| oc.serialized_size()).collect();
        let index = (0..final_opcode_sizes.len()).map(|i| final_opcode_sizes[0..i].iter().sum());
        let final_index = index.enumerate().map(|(a, b)| (b, a)).collect();
        let final_text = self.text.iter().flat_map(|c| c.to_bytes()).collect();

        (
            ByteCode {
                text: final_text,
                data: self.data.into_iter().collect(),
            },
            DebugInformation {
                ast_ids: self.ast_ids.into_iter().collect(),
                index: final_index,
            },
        )
    }
}

impl DataTypeInfo {
    /// If this is an owning heap ref, set it to disowned, otherwise panic
    pub fn disown(&mut self) {
        if let DataTypeInfo::HeapTypeInfo {
            dtype,
            owner_idx: owner_idx @ Owner::None,
        } = self
        {
            *owner_idx = Owner::Disowned;
        } else {
            panic!("calling disown on invalid target");
        }
    }
}
