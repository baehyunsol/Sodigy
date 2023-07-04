# Sodigy

Sodigy is a very abstracted, purely functional programming language.

It's still under development. Only parser and lexer are (partially) complete.

## Functions

- Every function in Sodigy is pure.
- Every function in Sodigy is evaluable at compile time.

## Types

Types in Sodigy are first-class objects. The type checker (which is not implmeneted yet) evaluates the type signatures in compile time, and calls `.is_subtype_of()`.

### Integers

Sodigy uses arbitrary-width integers.

### Numbers

Sodigy doesn't use floating points, but rational numbers.