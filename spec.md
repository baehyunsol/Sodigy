## Blocks

In Sodigy, everything is a block. A block consists of zero or more declarations (`let`, `func`, `struct` and `enum`), and exactly one expression at the end.

A block is wrapped by curly braces. A file is also a giant block, but it's not wrapped in curly braces.

## Decorators

A decorator decorates `let`, `func`, args of `func`, `struct`, fields of `struct`, `enum` and variants of `enum`. There are 2 types of decorators:

1. `@public` (no arguments)
2. `@test.eq((3, 4), 5)` (with arguments)

## Pattern matching

1. `match foo() { (0..3, _) => 1, ($x, $y) => x + y }`
2. `let pat ($x, $y) = foo();`
3. `if pat (0..3, _) = foo() { 1 } else { 0 }`

## Lambda Functions

- `\(a, b) => a + b`
- `\(a: Int, b: Int): Int => a + b`

## Literals

### String literals

A string literal starts with N (odd number) double-quotes, and ends with the same number of double-quotes.

- Binary Strings: prefix `b`.
- Formatted Strings: prefix `f`.
- Raw Strings: prefix `r`.
  - All the escapes are ignored.
  - If it's an expression, it's just a string. If it's a pattern, it's treated as a regex pattern.
    - You can bind the matched regex. The bound value is a tuple of `Option(String)`.
    - For example, value `m` in pattern `m @ r"(\d+)x(\d+)"` has type `(Option(String), Option(String), Option(String))`: 1 for the entire match and 2 for the groups.
- Mixes
  - You can use multiple prefixes at once (only `br`, `rb`, `fr`, `rf`).

### Char literals

A char literal starts with a single-quote character and ends with a single-quote character

- Binary Chars: prefix `b`
