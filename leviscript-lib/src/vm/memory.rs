use crate::core::*;
use crate::vm::Heap;

pub type Stack = Vec<RuntimeData>;
/// Represents the 3 relevant memory areas of the VM: Heap, Stack, and Data segment
#[derive(Default, Debug)]
pub struct Memory {
    pub stack: Stack,
    pub heap: Heap<RuntimeValue>,
    pub data_seg: Vec<ComptimeValue>,
}

pub enum Storable {
    OnHeap(RuntimeValue),
    OnStack(RuntimeData),
}

impl From<Vec<ComptimeValue>> for Memory {
    fn from(value: Vec<ComptimeValue>) -> Self {
        Self {
            stack: vec![],
            heap: Heap::new(),
            data_seg: value,
        }
    }
}

impl Memory {
    /// puts something on the stack
    pub fn push_stack<T>(&mut self, entry: T)
    where
        T: Into<RuntimeData>,
    {
        self.stack.push(entry.into());
    }

    /// puts a value onto the heap, and puts a reference to that value onto the stack
    pub fn push_heap<T>(&mut self, entry: T)
    where
        T: Into<RuntimeValue>,
    {
        let heap_idx = self.heap.push(entry.into());
        unsafe {
            let ptr: *const RuntimeValue = self.heap.get(heap_idx);
            self.stack.push(Data::Ref(RuntimeRef::HeapRef(ptr)));
        }
    }

    pub unsafe fn push_data_section_ref(&mut self, idx: usize) {
        let ptr: *const ComptimeValue = self.data_seg.get(idx).unwrap();
        self.stack.push(Data::Ref(RuntimeRef::DataSecRef(ptr)));
    }

    pub unsafe fn copy_stack_entry_to_top(&mut self, idx: usize) {
        let val = self.stack.get_unchecked(idx).clone();
        self.stack.push(val);
    }

    pub fn store(&mut self, storable: Storable) {
        match storable {
            Storable::OnHeap(x) => self.push_heap(x),
            Storable::OnStack(x) => self.push_stack(x),
        }
    }
}

impl<T> From<T> for Storable
where
    T: Into<CopyValue>,
{
    fn from(value: T) -> Self {
        Storable::OnStack(Data::CopyVal(value.into()))
    }
}

impl From<String> for Storable {
    fn from(value: String) -> Self {
        Storable::OnHeap(Value::Str(value))
    }
}
