## Blocks

In Sodigy, everything is a block. A block consists of zero or more declarations (`let`, `func`, `struct` and `enum`), and exactly one expression at the end.

A block is wrapped by curly braces. A file is also a giant block, but it's not wrapped in curly braces.

## Structs

Unlike Rust, it has `=` and `;`. That's because it's a declaration.

```
struct Person = {
    name: String,
    age: Int,
};
```

You can also declare a generic struct.

```
struct Message<T> = {
    id: Int,
    content: T,
};
```

## Decorators

A decorator decorates `let`, `func`, args of `func`, `struct`, fields of `struct`, `enum` and variants of `enum`. There are 2 types of decorators:

1. `@public` (no arguments)
2. `@test.eq((3, 4), 5)` (with arguments)

## Pattern matching

1. `match foo() { (0..3, _) => 1, (x, y) => x + y }`
2. `let pat (x, y) = foo();`
3. `if pat (0..3, _) = foo() { 1 } else { 0 }`

### Patterns

- Integer patterns
  - `0`
  - `0..5`, `0..=5`, `0..`, `..5`, `..=5`: a range of integers
  - `a`: matches any integer and bind name `a`
- String patterns
  - `""`: an empty string
  - `"abc"`: a string literal `"abc"`
  - `"abc" ++ a ++ "def"`: a string that starts with `"abc"` and ends with `"def"`, and bind name `a` to the slice `[3..-3]` of the string
- Regex patterns
  - A string literal prefixed with `r` (raw string) is treated as a regex pattern.
  - The string must match the entire regex pattern!
  - For example, `r"\d"` matches if the string is a single numeric character.
  - You can also bind names. For example, a pattern `(?<number>\d+)` matches a number and binds name `number` to the matched string.
    - Bound name always has type `Option<String>`. That's because the compiler is not smart enough to infer whether the group is optional or not.
- List patterns
  - `[]`: an empty list
  - `[a, b]`: a list with 2 elements, and bind names `a` and `b` to the elements
  - `[a] ++ _`: a list with at least 1 element, and bind name `a` to the first element
  - `[_] ++ a ++ [_]`: a list with at least 2 elements, and bind name `a` to the slice `[1..-1]` of the list
  - `[1..5, a, _]`: you can use other patterns inside a list pattern
- Tuple patterns
  - `()`: an empty tuple
  - `(a, b, _)`: a tuple with 3 elements, and bind names `a` and `b` to the first and the second elements
  - You cannot concat (`++`) tuples.
- Name bindings
  - `a @ 0..5`: matches an integer between 0 and 5, and bind name `a`
  - `a @ [_, _]`: matches a list with 2 elements, and bind name `a`

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
- Mixes
  - You can use multiple prefixes at once (only `br`, `rb`, `fr`, `rf`).

### Char literals

A char literal starts with a single-quote character and ends with a single-quote character

- Binary Chars: prefix `b`

## Type annotations

Syntactically, type annotations are always optional. It won't throw any syntax error for missing type annotations. But, it's a compile error if the inference engine cannot infer the type.

- `Int`: built-in integer type
- `List<Int>`: built-in list type, it has 1 argument
- `Fn<(Int, Int): Int>`: a function that takes 2 integers and returns 1 integer
