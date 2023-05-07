# Heap Management

At runtime a command like strcat can use the memory object to store data on the
heap. There is also an Opcode to delete something on the heap.

During compilation there are two mappings:

1. Scope to variables that live within it
2. variable name to heap id

When a scope is closed, at compile time, those mappings are used to generate
the opcodes to delete the vars on the heap that become invalid. Unless the
colapsing scope returns something, which, in this case, is first removed from
the list of dying entries.

At compile time, the same heap data structure as during runtime is used to
track the IDs the variables will have. Since a function like strcat should be
implemented as builtin, and be wrapped automatically by the proc-macro that is
yet to be written, it is neccessary to deduct the changes in stack and heap
from the function signature. 

As all arguments are put on the stack before the function call, and removed
during the function call, the net-change is that the result will go to the
stack, and if it's not a primitive it will also go to the heap.


