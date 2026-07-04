# Task

Write a tiny C compiler, named "mcc" in Rust.

The compiler should support compile C sources to RISCV64 assemblies.

For testing, "zig" and system llvm tools may be used to providing a cross-compiled libc and perform linking.
Qemu user-mode emulators may used to run the program.
