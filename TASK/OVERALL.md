# Task

Write a tiny compiler, named "mcc" in Rust.

The compiler should support compile "monkey C" sources to RISCV64 assemblies.
The filename extension of "monkey C" sources is ".mc".

"Monkey C" language is essentially a subset of C language with different syntax.
In addtion, "Monkey C" uses the same compilation model with C language.
Each file is a compilation unit, which should be compiled to an object file,
then all object files, plus additional libraries, are linked together.
The linking process could use a conventional linker, like "ld" or "ld.lld".

See [Monkey C Introduction](monkey-c-intro.md) for a concise description of the "monkey C" language.

In testing, the agent should:
1. Use system llvm tools to perform assemblying and linking.
2. Use qemu user-mode emulators to run the program.

All testcases should be in "testcases" directory.
