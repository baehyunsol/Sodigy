## Blocks

In Sodigy, everything is a block. A block consists of zero or more declarations (`let`, `func`, `struct` and `enum`), and exactly one expression at the end.

A block is wrapped by curly braces. A file is also a giant block, but it's not wrapped in curly braces.

## Decorators

A decorator decorates `let`, `func`, args of `func`, `struct`, fields of `struct`, `enum` and variants of `enum`. There are 2 types of decorators:

1. `@public` (no arguments)
2. `@test((3, 4), 5)` (with arguments)
