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
2. `let (x, y) = foo();`
3. `if let (0..3, _) = foo() { 1 } else { 0 }`

### Patterns

- Integer patterns
  - `0`
  - `0..5`, `0..=5`, `0..`, `..5`, `..=5`: These match a range of integers.
  - `a`: It matches any integer and bind name `a`.
- String patterns
  - `""`: It matches an empty string.
  - `"abc"`: It matches a string literal `"abc"`.
  - `"abc" ++ a ++ "def"`: It matches a string that starts with `"abc"` and ends with `"def"`, and bind name `a` to the slice `[3..-3]` of the string.
- Regex patterns
  - A string literal prefixed with `r` (raw string) is treated as a regex pattern.
  - The string must match the entire regex pattern!
  - For example, `r"\d"` matches if the string is a single numeric character.
  - You can also bind names. For example, a pattern `(?<number>\d+)` matches a number and binds name `number` to the matched string.
    - Bound name always has type `Option<String>`. That's because the compiler is not smart enough to infer whether the group is optional or not.
- List patterns
  - `[]`: It matches an empty list.
  - `[a, b]`: It matches a list with 2 elements, and bind names `a` and `b` to the elements.
  - `[a] ++ _`: It matches a list with at least 1 element, and bind name `a` to the first element.
  - `[_] ++ a ++ [_]`: It matches a list with at least 2 elements, and bind name `a` to the slice `[1..-1]` of the list.
  - `[1..5, a, _]`: You can use other patterns inside a list pattern.
- Tuple patterns
  - `()`: It matches an empty tuple.
  - `(a, b, _)`: It matches a tuple with 3 elements, and bind names `a` and `b` to the first and the second elements.
  - You cannot concat (`++`) tuples.
- Struct patterns
  - `Person { age: 25..=30, name }`: It matches an instance of `Person` whose age is between 25 and 30 (inclusive), and bind name `name` to the field `name` of the person.
- Tuple-struct patterns
  - `Point(x, y)`: It matches any point, and bind name `x` and `y` to its fields.
- Name bindings
  - `a @ 0..5`: It matches an integer between 0 and 5, and bind name `a`.
  - `a @ [_, _]`: It matches a list with 2 elements, and bind name `a`.
- Equality checks
  - `$a`: It matches if the value is equal to `a`. There must be a value named `a` in the name scope.
  - `[$a] ++ _`: It matches a list whose first element is equal to `a`.
  - `[b @ $a] ++ _`: It matches a list whose first element is equal to `a`, and bind name `b` to the first element.

## Lambda Functions

- `\(a, b) => a + b`
- `\(a: Int, b: Int) -> Int => a + b`

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
- `Fn(Int, Int) -> Int`: a function that takes 2 integers and returns 1 integer

## Assertions

```sodigy
assert two == 2;
let two = 1 + 1;

// By default, assertions are only enabled in debug-mode.
// With `@always` decorator, it's always enabled.
@always
assert 1 + 1 == 2;

// `@name` and `@note` will improve readability of the test result.
@name("add_test")
@note("It makes sure that `add` function is correct")
assert add(1, 1) == 2;
fn add(x: Int, y: Int): Int = x + y;
```

## IO (and impure functions)

### panic

```
fn panic(message: String);
```

`panic` is the only impure function that can be used anywhere in Sodigy. It prints the error message to stderr and the process is terminated with a non-zero exit code.

You can NEVER catch a panic. `panic` is impure, but catching a `panic` is even more impure.
