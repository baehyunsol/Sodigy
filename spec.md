## Blocks

In Sodigy, everything is a block. A block consists of zero or more declarations (`let`, `func`, `struct` and `enum`), and exactly one expression at the end.

A block is wrapped by curly braces. A file is also a giant block, but it's not wrapped in curly braces.

## Decorators

A decorator decorates `let`, `func`, args of `func`, `struct`, fields of `struct`, `enum` and variants of `enum`. There are 2 types of decorators:

1. `@public` (no arguments)
2. `@test((3, 4), 5)` (with arguments)

## Pattern matching

1. `match foo() { (0..3, _) => 1, ($x, $y) => x + y }`
2. `let pat ($x, $y) = foo();`
3. `if pat (0..3, _) = foo() { 1 } else { 0 }`

## Literals

### String/Char literals

- Normal Strings
  - Double quote, no prefixes.
  - It's supposed to be the same as rust's string literals.
- Normal Chars
  - Single quote, no prefixes.
  - It's supposed to be the same as rust's char literals.
- Binary Strings
  - Double quote, prefix `b`.
- Binary Chars
  - Single quote, prefix `b`.
- Formatted Strings
  - Double quote, prefix `f`.
- Raw Strings
  - It starts with an odd number (N >= 3) of double quotes, and ends with the same number of double quotes.
  - There's no escape characters in the literal, backslashes and double quotes are treated like other characters.
- Regex Strings
  - Double quote, prefix `r`.
  - If a backslash character is not followed by `\` or `"`, it's treated like other characters.
  - If it's an expression, it's just a string. If it's a pattern, it's treated specially.
    - You can bind the matched regex. The bound value is a tuple of `Option(String)`.
    - For example, value `m` in pattern `m @ r"(\d+)x(\d+)"` has type `(Option(String), Option(String), Option(String))`: 1 for the entire match and 2 for the groups.
- Mixes
  - You can create a prefixed raw string by using prefix + multiple quotes.
  - You can use multiple prefixes at once (only `br`, `rb`, `fr`, `rf`).
