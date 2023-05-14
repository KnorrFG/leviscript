# Type checking and casting

During compilation, it will be checked whether the type of an expression
matches the expected parameter of a call. 
If it doesn't the compiler will look up, whether there exists an appropriate
casting rule. If so, it will apply that rule, otherwise it will terminate
compilation with an error.

For casting rule lookup we have a compile-time dict, aka function from 
`(type_l, type_r)` to `Option<Opcode>`. The opcode will stand for a builtin that
expects the topmost stack argument to be of `type_l` and convert it to `type_r`

Types aren't a homogeneous mass. E.g. primitive types can be compared for
compatibility. But a type union, the any type, or restricted type may accept
multiple other types (also records can accept records that are super set). So
types needs a `is_satisfied_by()` function.
