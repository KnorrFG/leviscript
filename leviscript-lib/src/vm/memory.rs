use crate::core::*;
use crate::vm::Heap;

pub type Stack = Vec<RuntimeData>;
/// Represents the 3 relevant memory areas of the VM: Heap, Stack, and Data segment
#[derive(Default, Debug)]
pub struct Memory {
    pub stack: Stack,
    pub heap: Heap<RuntimeValue>,
    pub data_seg: Vec<ComptimeValue>,
    pub registers: [RuntimeData; 1],
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
            registers: Default::default(),
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

    /// returns refs to the stack from the back. 0 is the last element,
    /// 1 is the second to last etc
    pub fn stack_back(&self, ridx: usize) -> &RuntimeData {
        &self.stack[self.stack.len() - 1 - ridx]
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

    pub fn stack_top_to_reg(&mut self, reg: u8) {
        self.registers[reg as usize] = self.stack[self.stack.len() - 1].clone();
    }

    pub fn read_reg(&mut self, reg: u8) {
        self.stack.push(self.registers[reg as usize].clone());
    }

    pub fn pop_free(&mut self) {
        let r = self.stack.pop().unwrap();
        let Data::Ref(RuntimeRef::HeapRef(addr)) = r else {
            panic!("Pop_free found: {:#?}", r);
        };
        unsafe { self.heap.free(addr) };
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
