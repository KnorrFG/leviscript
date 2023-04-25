# LeviScript

Welcome to LeviScript. This is under development and far from useful. Once I'm
done, it will be a scripting language suitable for tasks that are typically
done by bash-scripts. It will also be an embeddable scripting language.
Statically typed, functional, and running on a VM.

This is the current project structure:

- The heavy lifting is done mostly in the leviscript-lib crate. I try to keep
  the documentation up to date, so `just doc-lib --open`[^1] is a good starting
  point.
- The interpreter crate uses the library to create the actual interpreter
  binary.
- the test-suite crate has tests for the scripting language, it compiles the
  interpreter, runs the test scripts, and checks whether they produce the
  expected output
- doc contains designs, manuals etc.

[^1]: [Just](https://github.com/casey/just) is a command runner. It simply
    executes the bash commands that are in the Justfile.]

