# Sodigy

Sodigy is a very abstracted, purely functional programming language.

It's still under development. Only parser and lexer are (partially) complete.

## Functions

- Every function in Sodigy is pure.
- Every function in Sodigy is evaluable at compile time.

### Decorators

Decorators decorate functions (and others WIP). For now, only built-in decorators are available. I don't have any plan for custom decorators in near future.

### Lambda Functions

The syntax of lambda functions is very simple: parameters and the body is inside a curly brace, and the curly brace follows a backslash (`\`). You may omit type annotations of parameters.

```
\{x: Int, y, x + y}
```

Above is an anonymous function that takes two integers and returns the sum of the integers.

Lambda functions can also capture its environment (closures).

```
def adder(n: Int): Func(Int, Int) = \{x: Int, x + n};

@test.eq(8)
def adder_test: Int = adder(5)(3);
```

### Constants vs 0-arg Functions

```
def PI_CONST: Number = 3.1415;
def PI_FUNC(): Number = 3.1415;
```

Both `PI_CONST` and `PI_FUNC` are valid. But they have 2 major differences. First, they're invoked in different way.

Second, `PI_CONST` is static, while `PI_FUNC` is not. It means that `PI_CONST` is evaluated only once, and cached by the runtime. If the code asks for `PI_CONST` multiple times, the runtime returns the memoized value.

## Values

### Block Expressions

Block expressions let you create lexical scopes. Its syntax is very similar to that of Rust and C/C++, but works very differently.

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

Unlike C/C++/Rust, values in a block are NOT evaluated in time-order. They're evaluated lazily. If a value is not used at all, it's not evaluated. If a value is used multiple times, it's evaluated only once and memoized (the memoized value is freed when the function exits).

Since the values are lazily evaluated, you cannot use them recursively. Belows are invalid.

```
{
    let x = 3;
    let y = 4;
    let x = 5;

    # Don't know which `x` to use
    x + y
}
```

```
{
    let x = z + 1;
    let y = x + 2;
    let z = y + 3;

    x + y + z
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

The above value is evaluated to `"3 + 4 = 7"`. It's just like Python!

#### Bytes

Byte literals are like that of Rust (as far as I know). A letter `b` followed by a string literal is bytes.

TODO: example

### `if` expressions

### `match` expressions

The syntax resembles that of Rust, except that it requires `$` before a name binding.

```
match foo() {
    Option.Some([$a, $b, ..]) => $a + $b + 1,  # more than 2 elements
    Option.Some([$a, $b]) => $a + $b,  # exactly 2 elements
    Option.Some([]) => 0,
    Option.Some(_) => -1,  # matches any
    Option.None => -2,
}
```

## Types

Types in Sodigy are first-class objects. The type checker (which is not implmeneted yet) evaluates the type signatures in compile time, and calls `.is_subtype_of()`.

### Integers

Sodigy uses arbitrary-width integers.

### Numbers

Sodigy doesn't use floating points, but rational numbers.

## Operators

### `` ` ``

`` ` `` is an infix operator, which modifies a value of a field.

Its lhs operand is the object you want to modify. Unlike the other infix operators, it has 2 rhs operands: the name of the field and the new value.

```
# TODO: add definition of `Person`

def set_age(p: Person, new_age: Int): Person = p `age new_age;

@test.eq(Person("Bae", 23))
def set_age_test: Person = set_age(
    Person("Bae", 21), 23
);
```

You can use whitespaces between `` ` `` and the name of the field, but I recommend you not to do so, for the sake of readability.

### `<>`

TODO: docs for concat operator

### `+>`

WIP: prepend operator

### `<+`

WIP: append operator

### `..`

TODO: docs for range operator

### `..~`

WIP: inclusive range operator

## Comments

`#` for a single line comment, and `##!` \~ `!##` for a multiline comment.

```
# This is a comment
def add_1_2: Int = 1 + 2;

##!
This is also a comment
!##
```
