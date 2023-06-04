# Implementing Functions

In Leviscript, functions are a complex thing, since they can be overloaded, and
value-matched, there are type-unions and stuff like that. There will be a
complex system that integrates all of that. In that system, the smalles unit,
which actually represents compiled code that is executed is called an
`fn_fragment`, and they are pretty similar to normal functions, with the only
special features being closures. Everything else is done by composing fragments

## Fn-Fragment

(if i use the word function in this section, it means fn-fragment)

There are three timepoints to keep in mind when compiling fragments: 
1. The time at which the fragment definition is compiled
1. The time at which code that calls the fragment is compiled
1. Runtime

Functions have closures, but those can  be empty. The VM holds a
`Vec<Option<Vec<RuntimeData>>>`. The Outer vec has one entry per defined
function, this inner vec has, if it exists, one field per captured variable.

During the execution of the function, the stack size is not known,
because it can be called from different places, with different stack states.
Therefore all stack access must be relative to the stack top, from where it can
be known. 

NOTE:
It might be a good idea to allocate all stack entries for a function at the
beginning, so the relative indices from the top don't change with the creation
of new variables.

Since fragment-definitions are expressions (or desugar to them) at the time of
definition, a fn ref must be put onto the stack.

### Compiling the Function definition

A function is compiled by a separate builder in function mode, so that all
stack lookups are relative to the stack top, the builder get's a symbol table
containing all args, local var, and closures.
The order on the stack is args, closures, local vars. Closures are coppied to
have better cache-locality.
At the end of a function, a bunch of instructions is written to store the
result, clean the stack, push back the result, and jump back to where the
program continues.

Additionally, in the builder that encountered the definition, code must be
generated to capture the closure data of the function, and store it in the
closure register of the VM. AND it needs to push a ref to that new fn onto the
stack.

### Compiling the fn-call

Calling a function must create text that does the following

1. Put the back jump addr on the stack (where to go after fn execution)
1. put all fn-args onto the stack
1. copy the closure data to the stack
1. allocate stack entries for all local vars.
1. jump to first instruction of fn


