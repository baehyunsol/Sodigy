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

Sodigy uses a software-implemented ratio type for real numbers. It's extremely slow, compared to hardware-implemented floating points.

If you want to do heavy computations... just don't use Sodigy.

```sodigy
assert 0.1 + 0.2 == 0.3;
assert 1.234e-5 == 0.00001234;
assert (2.89).sqrt() == 1.7;
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

### `as`, `as?` and `as!`

`as` is a type conversion operator. You can convert a type of a value as long as it implements `std.convert.convert`. The syntax is `value as <type>`. Note that the type must be in angle brackets.

```sodigy
let x = 3;

assert x as <Number> == 3.0;
```

You can use `as?` for type conversions that may fail. It'll call `std.convert.try_convert`.

```sodigy
let x = "3";

assert x as? <Int> == Ok(3);
```

Let's say types A and B implements `try_convert` but not `convert` (e.g. `String` to `Int`). In many cases, you already did the safety check and you can just unwrap the result. Instead of writing `(value as? <type>).unwrap()`, you can use `as!` operator. `x as! <T>` is a shorthand for `(x as? <T>).unwrap()`.

```sodigy
let x = "3";

// These 2 are the same.
assert (x as? <Int>).unwrap() == 3;
assert x as! <Int> == 3;
```

## Functions

Use `fn` keyword to define a function. The keyword is followed by a name of the function, a list of parameters (a parenthesis), an assignment (`=`), its return value, and a semicolon.

```sodigy
assert add(3, 4) == 7;
fn add(x, y) = x + y;

assert three() == 3;
fn three() = 3;
```

In sodigy, type annotations are (almost) always optional. Adding a type annotation might make your code more readable. Use `->` to state the return type of the function.

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
    Error(E),
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

## Pipelines

Sodigy has a pipeline operator (`|>`), which allows you to call a series of functions. The result of lhs is passed to rhs. Unlike gleam or bash, you have to explicitly pass the result with `$` sign.

```sodigy
fn add1(x) = x + 1;

// `$` is the result of the left hand side of `|>`.
// `1 |> add1 |> add1` is a syntax error.
let three = 1 |> add1($) |> add1($);
assert three == 3;
```

You must pass the result to the right hand side. If you don't, that's a syntax error.

```sodigy, compile_error
// The value in the left hand side (100) is not used by anyone. It's a syntax error.
let x = 100 |> 200;
```

You can nest pipelines. In a nested pipeline, `$` references the closest (inner-most) pipeline.

```sodigy
fn add1(x) = x + 1;

let x = 1
    |> add1($)  // `$` refers to 1

    // the first `$` refers to `add1($)` in the previous line, and
    // the second `$` refers to `$ * 2` in this line.
    |> add1($ * 2 |> add1($));

assert x == 6;
```

You can even use piped values in patterns (TODO: document).

## Pattern Matchings

Sodigy has a very flexible and expressive pattern matching system. The syntax resembles that of Rust. The biggest difference is that you have to put a dollar sign (`$`) in front of a name to bind a name.

Use `match` keyword to match a pattern. The keyword is followed by a value, and curly braces. The curly braces contain match arms.

```sodigy
fn to_string(n: Int) -> String = match n {
    ..0 => "negative number",
    0 => "zero",
    1 => "one",
    2 => "two",

    // `_` is a wildcard. It matches every value.
    _ => "very large number",
};

assert to_string(-1) == "negative number";
assert to_string(0) == "zero";
assert to_string(1) == "one";
assert to_string(2) == "two";
assert to_string(100) == "very large number";
```

```sodigy
fn greet(name: String) -> String = match name {
    "Bae" => "Hi, Bae",

    // You have to use a dollar-sign (`$`) for a name binding.
    $name @ ("John" | "Jane") => f"Good to see you, {name}",
    $other => f"Hello, {other}",
};

assert greet("Bae") == "Hi, Bae";
assert greet("John") == "Good to see you, John";
assert greet("Park") == "Hello, Park";
```

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

```sodigy, compile_error
#[poly]
fn first<T, U>(x: T, y: U) -> T;

#[impl(first)]
fn first_int(x: Int, y: String) -> String = y;
```

## Associated functions

## Tests

You can use `assert` keyword to make assertions. When you run `sodigy test`, it'll run all the top-level assertions in the source code.

```sodigy
let x = 3;
let y = 4;

// Run `sodigy test` to run this assertion.
assert x + y == 7;
```

If `assert` is inside a block, the assertion is run whenever the block is evaluated.

```sodigy
// Everytime you call `foo`, it'll check if `x` is greater than 0.
fn foo(x) = {
    assert x > 0;
    x + 1
};
```

You can add a name and a note to an assertion using `#[name]` and `#[note]` decorator.

`#[name]` is evaluated at compile time, so it must be a string literal. `#[note]` is evaluated at runtime, so it can be any string expression.

```sodigy, run_error
fn foo(x) = {
    #[name("foo-x-greater-than-0")]
    #[note(f"expected `x` to be greater than 0, but `x` is {x}.")]
    assert x > 0;

    x + 1
};

assert foo(-1) == 0;
```

By default, assertions are run only in the test mode (`sodigy test`). If you build the program (`sodigy run` or `sodigy build`), the assertions are all gone.

If there's an important assertion that has to be run in production, use `#[always]` decorator.

```sodigy, compile_error
fn foo(x, y) = {
    // This will run only in the test mode.
    assert less_important_check(x, y);

    // This will always run, even in production.
    #[always]
    assert very_important_check(x, y);

    x + y
};
```

## Decorators

TODO: documentation

## Effects

Sodigy is an effectful-language! By default, every function (defined with `fn` keyword) is pure. But sometimes you need effectful functions (e.g. file IO, time, random, ...).

There are 2 effects in Sodigy:

- `ndet` (non-deterministic): This function may return different values even if the same inputs are given.
- `proc` (procedure): When you call a procedure, its behavior is observable in the outside world.

Here's how I (informally) define `proc`: If a return value of `fn` or `ndet fn` is not used, the function won't be called and that's perfectly fine. But if a return value of `proc` or `ndet proc` is not used, we still have to call the procedure because not calling the procedure will change the behavior of the program.

```sodigy
// This is a non-deterministic function.
use std.random.random_int;

// In a non-deterministic function, you can call another n-det function.
ndet fn random_numbers() -> [Int] = [random_int(), random_int()];

// This is a compile error.
// fn random_numbers() -> [Int] = [random_int(), random_int()];

// This is not a compile error, but a compile warning because you're
// not calling any ndet function inside a ndet function.
ndet fn empty_list() -> [Int] = [];
```

Inside a `proc` context, you can use `do` keyword. With the `do` keyword, you can execute a procedure. The compiler guarantees that `do` keywords are not ignored, and their order does not change.

```sodigy
// This is a procedure.
use std.time.sleep;

// Inside a `proc` context, you can use the `do` keyword!
// It'll execute the procedure and discard the result.
proc sleep_and_return(t, n) = {
    do sleep(t);
    n
};
```

Functions with different effects have differet types. See the type annotations below:

```sodigy
use std.random.random_int;
use std.time.sleep;

let pure_func: Fn(Int) -> Int = \n => n + 1;
let ndet_func: NdetFn(Int) -> Int = ndet \n => random_int() + n;
let det_proc: Proc(Int) -> Int = proc \n => { do sleep(0.1); n + 1 };
let ndet_proc: NdetProc(Int) -> Int = ndet proc \n => { do sleep(0.1); random_int() + n };
```

Lambdas can be effectful, too. Use `ndet` and/or `proc` keyword before the backslash.

```sodigy
use sodigy.random.random_int;
use sodigy.time.sleep;

let ndet_lambda = ndet \=> [random_int(), random_int()];

// This is a compile error because there's no `ndet` keyword.
// let ndet_lambda = \=> [random_int(), random_int()];

// This lambda is a non-deterministic procedure.
// You have to put the `ndet` keyword before the `proc` keyword.
let ndet_proc_lambda = ndet proc \=> { do sleep(0.5); random_int() };
```

Creating an effectful function is not effectful. It's effectful only if you call the effectful function.

```sodigy
use sodigy.random.random_int;

// This is pure.
let impure_functions: [Ndet() -> Int] = [
    random_int,
    ndet \() => random_int(),
];

// This is impure.
// let impure_function_call = impure_functions[0]();
```

### panic / exit

Sodigy doesn't treat `panic` as an impure function, so you can use this function anywhere.

You can NEVER catch a panic. `panic` is impure, but catching a `panic` is even more impure.

Also note that `exit` is an impure function. My intention is that 1) you `panic` when something goes really wrong and there's nothing you can do and 2) you `exit` when everything's successful and you want to terminate the program.

## Macros

- `include_string!(path: String) -> String`
- `include_bytes!(path: String) -> Bytes`
- `type_name!(t: Type) -> String`
- `type_name_of_value!(v: Expr) -> String`
- `number_of_variants!(t: Type) -> Int`
- `number_of_fields!(t: Type) -> Int`
- `name_of_variants!(t: Type) -> [String]`
- `name_of_fields!(t: Type) -> [String]`
- `file!() -> String`
- `module_path!() -> String`
- `line!() -> Int`, `column!() -> Int`
