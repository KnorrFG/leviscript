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
#[derive(Debug)]
pub enum DataTypeInfo {
    DataSecTypeInfo { dtype: HeapType, dsec_idx: usize },
    HeapTypeInfo { dtype: HeapType, is_owner: bool },
    StackType(StackType),
    CallableType(CallableType, Box<Signature>),
}

impl Clone for DataTypeInfo {
    fn clone(&self) -> Self {
        match self {
            DataTypeInfo::DataSecTypeInfo {
                dtype: t,
                dsec_idx: i,
            } => DataTypeInfo::DataSecTypeInfo {
                dtype: t.clone(),
                dsec_idx: *i,
            },
            DataTypeInfo::HeapTypeInfo { dtype, .. } => DataTypeInfo::HeapTypeInfo {
                dtype: dtype.clone(),
                is_owner: false,
            },
            DataTypeInfo::StackType(t) => DataTypeInfo::StackType(t.clone()),
            DataTypeInfo::CallableType(t, s) => DataTypeInfo::CallableType(t.clone(), s.clone()),
        }
    }
}

impl DataTypeInfo {
    pub fn str() -> Self {
        Self::HeapTypeInfo {
            dtype: HeapType::Str,
            is_owner: true,
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
        self.stack_info
            .push_back(self.stack_info[*entry_idx].clone());
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

    /// removes the n topmost entries from the stack_info. Does not modify text
    pub fn pop_stack_entries(&mut self, n: usize) {
        for _ in 0..n {
            self.stack_info
                .pop_back()
                .expect("stack was emtpy unexpectedly");
        }
    }

    pub fn adjust_memory_for_call(
        &mut self,
        sign: &Signature,
        n_var_args: Option<usize>,
        ast_id: usize,
    ) {
        self.pop_stack_entries(sign.args.len());
        if sign.var_arg.is_none() {
            assert!(
                n_var_args.is_none(),
                "n_var_args provided for fn signature without variadig argument"
            );
        } else {
            let Some(n) = n_var_args else {
                panic!("function has variadic arguments, but n_var_args is None");
            };
            // n - args + the entry that tells the number of args
            self.pop_stack_entries(n + 1);
        }
        self.create_value_in_memory(&sign.result, ast_id);
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
                    is_owner: true,
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
