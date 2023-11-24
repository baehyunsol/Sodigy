# Sodigy

Purely functional Rust-like programming language.

It's still under development. Only parser and lexer are (partially) complete.

The goal of this language is to help programmers implement their idea as fast as possible. (not to run fast, but to implement fast).

## Functions

- Every function in Sodigy is pure.
  - Sodigy (and many other purely functional languages), doesn't consider terminating the entire program as an impure behavior. That's why `panic`, `assert` and many other debug functions are pure functions.
- Every function in Sodigy is evaluable at compile time.

### Decorators

Decorators can decorate almost everything: functions, enums, structs, enum variants, struct fields, and function arguments.

| shape                         | applied to    | what it does                     |
|-------------------------------|---------------|----------------------------------|
| `test.eq(val)`                | function `f`  | asserts that `f == val` or `f() == val`.    |
| `test.expected(args, value)`  | function `f`  | asserts that `f(args) == value`, `args` is a tuple of arguments.  |
| `test.false`                  | function `f`  | alias for `test.eq(Bool.False)`.  |
| `test.true`                   | function `f`  | alias for `test.eq(Bool.True)`.   |
| `test.before(lambda)`         | function `f`  | `lambda` is called everytime `f` is called. It's called before `f`. The lambda may capture `f`'s arguments. |
| `test.after(lambda)`          | function `f`  | It's like `test.before`, but the lambda takes one input: the output of `f`. |

```
# A decorator decorates the following function.
# A decorator is not followed by a semi colon.
@test.eq(4)
let add_test = 2 + 2;

# Multiple decorators may decorate a function.
@test.eq(Bool.True)
@test.true
let add_test2 = 2 + 2 == 4;

# It makes sure that `x` is even.
# Don't forget to use `assert_eq`. Otherwise, the lambda wouldn't have any effect.
@test.before(\{assert_eq(x % 2, 0)})
let foo(x: Int) = x + 1;

# It makes sure that `bar` always returns an odd number.
@test.after(\{ret, assert_eq(ret % 2, 1)})
let bar(x: Int) = foo(x);
```

### Lambda Functions

The syntax of lambda functions is very simple: parameters and the body is inside a curly brace, and the curly brace follows a backslash (`\`).

```
\{x: Int, y, x + y}
```

Above is an anonymous function that takes two integers and returns the sum of the integers.

Lambda functions can also capture its environment (closures).

```
let adder(n: Int): Func(Int, Int) = \{x: Int, x + n};

@test.eq(8)
let adder_test: Int = adder(5)(3);
```

### Constants vs 0-arg Functions

```
let PI_CONST: Number = 3.1415;
let PI_FUNC(): Number = 3.1415;
```

Both `PI_CONST` and `PI_FUNC` are valid. But they have 2 major differences. First, they're invoked in different way.

Second, `PI_CONST` is static, while `PI_FUNC` is not. It means that `PI_CONST` is evaluated only once, and cached by the runtime. If the code asks for `PI_CONST` multiple times, the runtime returns the memoized value.

## Values

### Scoped block expressions

Scoped block expressions let you create lexical scopes. Its syntax is very similar to that of Rust and C/C++, but works very differently.

```rust
{
    let x = 3;
    let y = 4;
    let z = {
        let x = 5;
        let w = 6;

        x + w
    };

    x + y + z
}
```

The above expression is evaluated to 18. Each block has their own scope, and a block must be evaluated to a value. The value comes at the end of a block. The value is not followed by a semi-colon, while the definitions of local values are.

Unlike C/C++/Rust, values in a block are evaluated lazily. If a value is not used at all, it's not evaluated. If a value is used multiple times, it's evaluated only once and memoized (the memoized value is freed when the function exits).

Since the values are lazily evaluated, you cannot use them recursively. Belows are invalid.

```
{
    let x = z + 1;
    let y = x + 2;
    let z = y + 3;

    x + y + z
}
```

Also, there's no shadowing. That means you cannot define values with the same name in a scope.

```
{
    let x = 3;
    let y = 4;
    let x = 5;

    # Don't know which `x` to use
    x + y
}
```

### String Literals

Sodigy doesn't distinguish double-quoted literals and single-quoted literals. It only has `String` type, but no `Char` type. It may change in the future.

Internally, Sodigy strings are lists of integers. Each integer represent a character, not a byte. For example, a korean character "한" is 3 bytes in UTF-8. It's `[237, 149, 156]` in Rust, and `"한".len()` is 3 in Rust. But in Sodigy, it's a single character, which is 54620 in integer. It makes Sodigy slower than Rust, but makes it much easier to deal with strings, especially indexing.

Beside normal string literals, there are two special ones: formatted strings and bytes.

#### Formatted strings

Formatted strings are like that of Python (as far as I know). A letter `f` followed by a string literal is a formatted string.

```
{
    let a = 3;
    let b = 4;

    f"{a} + {b} = {a + b}"
}
```

The above value is evaluated to `"3 + 4 = 7"`. Like in Python!

#### Bytes

Byte literals are like that of Rust (as far as I know). A letter `b` followed by a string literal is bytes.

TODO: example

### `if` expressions

`if` is an expression in Sodigy.

```
let x = if cond() { 3 } else { 4 };
```

It'd be very familiar if you know Rust/Haskell/Elixir, or any other functional language. If you're from C/C++, you must be familiar with ternary operators. That's an `if` expression.

It also supports Rust-like `if let` syntax.

### `match` expressions

The syntax resembles that of Rust, except that it requires `$` before a name binding.

```
match foo() {
    Option.Some([$a, $b, $c, ..]) => $a + $b + 1,  # more than 2 elements
    Option.Some([$a, $b]) => $a + $b,  # exactly 2 elements
    Option.Some([]) => 0,
    Option.Some(_) => -1,  # matches any
    Option.None => -2,
}
```

## Types

Types in Sodigy are first-class objects. The type checker (which is not implmeneted yet) evaluates the type signatures in compile time, and calls `.is_subtype_of()`.

### Type annotations

You can use type annotations in many places. The annotations are always optional, and the compiler tries to infer the type. When the compiler cannot infer the type, it would ask you for an annotation. Then you have to provide one.

```
# Compiler can figure out the type of `PI` without annotation.
let PI = 3.1415;

# Compiler can figure out the type of `x` without annotation.
let foo(x): Int = x + 1;
```

### Integers

Sodigy uses arbitrary-width integers.

### Numbers

Sodigy doesn't use floating points, but rational numbers.

### Enums

Enums in Sodigy are like those of Rust. There are a few differences when providing type parameters.

```
let enum Option<T> = {
    None,
    Some(T),
};
```

```
Option.None            # valid
Option.Some(5)         # valid
Option(Int).Some(5)    # valid expression, invalid pattern
Option(Int).None       # valid expression, invalid pattern
Option(Int).Some("abc")     # type error
Option.Some(Int)       # valid, but the type is `Option(Type)`, not `Option(Int)`
Option.Some(Int, 5)    # valid
Option.Some(Int, "abc) # type error
```

### Struct

Structs in Sodigy are like those of Rust.

```
let struct Person = {
    name: String,
    age: Int,
};
```

## Operators

### `` ` ``

You can make an infix-operator using `` ` ``. An operator is `` ` `` followed by an identifier without whitespace. The operator modifies a value of a field. The identifier is the name of the field that you want to modify. See how `` `age `` works below.

```
struct Person {
    age: Int,
    name: String,
}

let set_age(p: Person, new_age: Int): Person = p `age new_age;

@test.eq(Person("Bae", 23))
let set_age_test: Person = set_age(
    Person("Bae", 21), 23
);
```

### `<>`

`<>` concatonates 2 lists or strings. You can also overload this operator (WIP).

```
@test.eq([1, 2, 3, 4, 5, 6])
let concat_test: List(Int) = [1, 2, 3] <> [4, 5, 6];
```

### `+>`

TODO: docs for prepend operator

### `<+`

TODO: docs for append operator

### `..`

`..` makes an exclusive range. For example, `1..4` is a range from 1 to 3, and `'a'..'c'` is `'a'` and `'b'`. An extra argument can set the step of the range. For example, `1..10..2` is `1`, `3`, `5`, `7`, and `9`. Negative steps are also possible.

You can index lists and strings with a range. For example, `a[0..3]` takes the first 3 elements of `a`. Or, `a[-3..]` takes the last 3 elements.

### `..~`

`..~` is like `..`, but includes the last index. For example, `1..~3` is `1`, `2` and `3`.

It's very useful in some cases. For example, if you want a pattern that covers lower case alphabets, it's either `'a'..'{'` or `'a'..~'z'`. The second one looks much better, doesn't it?

## Comments

`#` for a single line comment, `##>` for a doc-comment and `##!` \~ `!##` for a multiline comment.

```
# This is a comment
let add_1_2: Int = 1 + 2;

##!
This is also a comment
!##

##> This function adds two numbers.
let add(x: Int, y: Int): Int = x + y;
```

## For Rust programmers
