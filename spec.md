# Sodigy

Sodigy is a purely-functional, Rust-like programming language.

## Installation

TODO: documentation

## Values

You can bind a name to a value using `let` keyword. There's no "variable" in Sodigy: everything is immutable.

A `let` statement must be followed by a semicolon (`;`). Type annotation is optional in Sodigy.

```sodigy
let three = 3;
let pi = 3.14;

let hundred: Int = 100;
```

## Data Types

### Integers

Sodigy uses an arbitrary-width integer type.

```sodigy
assert 1_000_000_000_000_000_000_000_000 + 1 == 1_000_000_000_000_000_000_000_001;
```

You can use underbar (`_`) characters in integer literals for readability, see the example above.

Like most languages, you can write integer literals with various bases.

```sodigy
// a hexadecimal literal
assert 0x3e8 == 1000;

// an octal literal
assert 0o1750 == 1000;

// a binary literal
assert 0b1111101000 == 1000;
```

### Real Numbers

TODO: documentation

```sodigy
assert 0.1 + 0.2 == 0.3;
assert 1.234e-5 == 0.00001234;
```

### Bytes

TODO: documentation

### Strings/Chars

Use double-quotes to specify a string literal and single-quotes for a character literal.

```sodigy
let s = "Hello, World!";
let c = 'a';
```

A character literal must be ... a single character, of course!

```sodigy, compile_error
// This is a syntax error.
let c = 'abc';

// So is it.
let e = '';
```

A string literal starts with N (odd number) double-quotes, and ends with the same number of double-quotes.

```sodigy
// a string literal with triple double-quotes.
let s = """
This is a string literal.
There's a double-quote here: "
This is still a string literal.
""";
```

String escape rules are almost identical to [Rust](https://doc.rust-lang.org/reference/tokens.html#r-lex.token.literal.str).

```sodigy
assert "\'" == "'";
assert "\u{ac00}" == "가";
assert "\n" == "
";
assert '\x41' == 'A';
assert "\x41\x42\x43" == "ABC";
```

Raw string literals ignore escapes. A raw string literal starts with `r` character, followed by a string literal.

Please note that using multiple double-quotes has nothing to do with it being a raw string literal. The number of the starting double-quotes determines how many double-quotes it needs to end the literal, and the `r` character determines whether it ignores escapes.

```sodigy
assert r"\" == "\\";
assert r"\x41" == "\\x41";
```

Internally, `String` is just a list of `Char`.

```sodigy
assert "Sodigy" == ['S', 'o', 'd', 'i', 'g', 'y'];
assert "" == [];
```

### Binary Strings/Chars

TODO: documentation

### Formatted Strings

TODO: documentation

```sodigy
let x = 3;
let y = 4;

assert f"{x} + {y} = {x + y}" == "3 + 4 = 7";
```

## Operators

### `++`

`++` concatonates two lists. Since `String` is `[Char]`, you can use the `++` operator with strings.

```sodigy
assert "Hello, " ++ "World!" == "Hello, World!";
assert [1, 2, 3] ++ [4, 5, 6] == [1, 2, 3, 4, 5, 6];
```

### `+>`

`+>` is a "prepend" operator. It prepends an element to a list.

```sodigy
assert 3 +> [4, 5, 6] == [3, 4, 5, 6];
assert 'a' +> "bcd" == "abcd";
```

### `<+`

`<+` is an "append" operator. It appends an element to a list.

```sodigy
assert [1, 2, 3] <+ 4 == [1, 2, 3, 4];
assert "abc" <+ 'd' == "abcd";
```

## Functions

Use `fn` keyword to define a function. The keyword is followed by a name of the function, a list of parameters (a parenthesis), an assignment (`=`), its return value, and a semicolon.

```sodigy
assert add(3, 4) == 7;
fn add(x, y) = x + y;

assert three() == 3;
fn three() = 3;
```

In sodigy, type annotations are always optional. Adding a type annotation might make your code more readable. Use `->` to state the return type of the function.

```sodigy
fn add(x: Int, y: Int) -> Int = x + y;

// The compiler will infer that `y` has type `Int`.
fn mul(x: Int, y) -> Int = x * y;
```

## Comments

Sodigy's comment syntax is very similar (or identical) to Rust/Zig/C's.

TODO: documentation

### Doc Comments

Sodigy's doc comment syntax is very similar (or identical) to Rust/Zig's.

TODO: documentation

## Structs

TODO: documentation

```sodigy
struct Person = {
    name: String,
    age: Int,
};
```

## Enums

TODO: documentation

```sodigy
enum Result<T, E> = {
    Ok(T),
    Err(E),
};
```

## Type Aliases

TODO: documentation

```sodigy
type OptionalInt = Option<Int>;
type ErroneousInt<E> = Result<Int, E>;

// This is how `String` is defined in std.
type String = [Char];
```

## Blocks

A block is an anonymous name scope. Any code wrapped in curly braces form a block. A block is always an expression. A block is evaluated to the last value in the block.

```sodigy
let x = {
    let a = 3;
    let b = 4;

    a + b
};

assert x == 7;
```

Blocks create their own scope. Names defined in a block cannot be accessed from outside.

```sodigy
let block = {
    fn add(x) = x + 10;
    assert add(10) == 20;

    let y = 10;
    assert y == 10;

    ()
};

fn add(x) = x + 100;
assert add(100) == 200;

let y = 100;
assert y == 100;
```

## Pattern Matchings

TODO: documentation

## Type Annotations

Syntactically, type annotations are always optional. It won't throw any syntax error for missing type annotations. But, it's a compile error if the inference engine cannot infer the type.

- `Int`: Built-in integer type.
- `[Int]`: Built-in list type.
- `(Int, Int)`: Built-in tuple type.
- `Option<Int>`: Option type, and it has 1 argument.
- `Fn(Int, Int) -> Int`: A function that takes 2 integers and returns 1 integer.
- `Result<_, _>`: You can omit a part of a type annotation.

## Generics

TODO: documentation

## More generics

NOTE: It's a dark magic. DO NOT USE THIS.

You can make generic functions even more generic with `#[poly]` decorator.

```sodigy
// A polymorphic generic doesn't require a body.
#[poly]
fn greet<T>(v: T) -> String;

#[impl(greet)]
fn greet_int(n: Int) -> String = f"Hello, integer {n}!";

#[impl(greet)]
fn greet_number(n: Number) -> String = f"Hello, number {n}!";

assert greet(3) == "Hello, integer 3!";
assert greet(3.0) == "Hello, number 3.0!";

#[note("You can explicitly call impls.")]
assert greet_int(3) == "Hello, integer 3!";

// This is a type error.
// assert greet_number(3) == "Hello, number 3!";
```

When you call `greet`, the compiler tries to find an implementation that matches the types of the arguments. If it can't find one, that's a type error. If it finds multiple, (TODO: what should I do?).

You can provide the default implementation of `poly`. If the compiler cannot find an implementation, it'll use the default implementation.

```sodigy
#[poly]
fn greet<T>(v: T) -> String = f"Hello, {v}!";

#[impl(greet)]
fn greet_int(n: Int) -> String = "Hello, number {n}!";

assert greet("World") == "Hello, World!";
assert greet(3) == "Hello, number 3!";
```

Since `greet` has type `Fn(T) -> String`, all its impls must return `String`. That saying, type signature of implementations of a poly generic must be compatible with the poly generic.

For example, below does not compile because `first` expects the return type and the first argument's type to be the same, but `first_int` takes `(Int, String)` as inputs and returns `String`.

```sodigy, assert_compile_error
#[poly]
fn first<T, U>(x: T, y: U) -> T;

#[impl(first)]
fn first_int(x: Int, y: String) -> String = y;
```

## Tests

TODO: documentation

## Decorators

TODO: documentation

## IO (and impure functions)

TODO: documentation

### panic

`panic` is the only impure function that can be used anywhere in Sodigy.

You can NEVER catch a panic. `panic` is impure, but catching a `panic` is even more impure.
