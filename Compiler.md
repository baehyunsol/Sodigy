# Sodigy Compiler

This is a document for the Sodigy Compiler. It's written in Rust, and there's no plan for bootstrapping. The compiler is largely incomplete. It doesn't generate machine code, and you can run very simple codes.

## Intermediate Representations

Like other compilers, Sodigy compiler goes through multiple steps to compile.

1. Code
  - It's a text file the programmer provides. There's no modification yet.
2. `Token`
  - Raw texts is processed by `sodigy_lex` to become `Token`s.
  - It's a very simple representation. It distinguishes identifiers, punctuations, and literals. That's it.
  - In this step, very simple compie errors are caught: like invalid utf-8 code, unterminated block comments, unterminated string literals, etc...
3. `TokenTree`
  - It's like `TokenTree` in rustc. It's a slightly more advanced version of `Token`.
  - Now it knows whether an identifier is a keyword or not.
  - It also groups tokens in this stage. It checks whether parenthesis are properly closed, and marks the start and end of parenthesis.
4. AST
  - In this step, the compiler starts dealing with the semantics.
5. High level Intermediate Representation (WIP)
  - It's not that different from AST, except that most names are resolved in this stage. Only names within the same file are resolved. Functions and constants that are used by multiple files are resolved and analysed at MIR stage.
  - HIR is the basic building block for incremental compilation. 99% of HIR can be built from a single file (except custom macros), and that makes it easy to reuse HIRs built at previous compilations.
6. Mid level Intermediate Representation (Not yet)
  - In this stage, all the names are fully resolved.
  - Now that it can find the origin of all the values, it checks and infers types. This stage is where most of the analysis are done.
7. Low level Intermediate Representation (Not yet)
8. Output (Not yet)
  - C code vs LLVM IR vs Cranelift vs Machine Code

## Interns

For performance reasons, all the literals are interned.

## Errors

In order to make the life of programmers easier, the compiler tries to emit as many error messages as possible. When an error is found, it doesn't stop the compilation immediately. It tries to continue analysis and compilation until it makes no sense at all. If you're adding a new error to the compiler, please make sure to keep this in mind. For example, let's say there's a name collision in a function. If you just stop the entire thing at that point, you're missing potential errors in other functions. There could be more!
