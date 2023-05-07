# Builtin Mechanics

There are two ways built ins could be handled:

1. Each built-in function gets its own opcode
2. There is one Opcode to execute a built-in function and as an argument it
   gets an identifier for the function. 

The downside of the second approach is that there are two dispatch steps:
run the built-in opcode, get the ID, and then exec fn based on the ID, as
opposed to: exec the function based on the opcode. This can't be optimized away
by the compiler either.

the downside of the first approach is that it represents the reality worse than
the second approach: there are things that are the same for all built ins, and
having them marked as built ins would be nice. 

Also, it would mean that I have to write the built-in fn, in the built-in module,
which is annotated with a proc macro, AND define an opcode, and maybe annotate
it with information that is already in the fn signature.
The code for exec_strcat (let's just use strcat as example here) must be
generated. Probably by the proc macro that annotates the built-in module

Ok, it is how it is now. The speed is probably worth the effort, I go with
approach number one.
