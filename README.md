# Sodigy

Purely functional Rust-like programming language.

It's still under development.

In order to build the compiler, read [this](Build.md).

## Goal of Sodigy (and the compiler)

- Compiler not only compiles the code, but also helps user write the code.
  - The compiler possesses the most comprehensive understanding of the code.
  - It should give very detailed error messages and does as much static-analysis as possible.
- The language has to be kept simple.
  - The core of language consists of a small set of instructions.
  - Libraries have to be written in Sodigy, instead of using C ffi.
- The language has to be abstracted.
  - `0.2 + 0.7` is `0.9`, no matter which machine you're using.
  - `factorial(21)` is `51090942171709440000`, not an integer overflow.
  - Sodigy completely separates side effects and functions.
- Everything has to be explicit.
  - Clarity should not be sacrificed for the sake of brevity.
