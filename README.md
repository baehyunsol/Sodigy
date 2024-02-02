# Sodigy

Purely functional Rust-like programming language.

It's still under development.

In order to build the compiler, read [this](Build.md).

## Goal of Sodigy

- Programmers can implement their idea as fast as possible.
  - It doesn't mean the result *RUNS* fast, but the programmers can get the result fast. Of course the runtime performance is important, but build-test cycle is much more important.
- The core of the language has to be as simple as possible.
  - Don't use built-in functions unless necessary. Write functions in Sodigy.

## `let` keywords

Use `let` keyword to bind a name to a value. Function definitions also use `let` keyword.

```
# constant
let PI = 3.1415;

# function
let add(x, y) = x + y;
```

Everything has type in Sodigy. You can annotate types like below.

```
let Answer: Int = 42;

let add(x: Int, y: Int): Int = x + y;
```

When type annotations are missing, the compiler tries to infer the type. When it cannot infer, it'll ask you for the annotation.

### Scoped `let`s

In sodigy, names have scopes. You can scope `let`s using curly braces. It's like that of `let..in` in Haskell, or scoped `let`s in Rust.

```
# it always returns `x + 1`
let add(x: Int, y: Int) = {
    let y = 1;

    x + y
};
```

But you cannot shadow names. That means you cannot bind the same name multiple times in a block. Below is invalid.

```
let add(x: Int, y: Int) = {
    let z = 1;

    # you cannot do this
    let z = 2;

    x + y + z
};
```

### `let pattern`

You can destruct patterns with `let pattern`.

```
let pattern ($x, $y) = (0, 1);
```

is equivalent to

```
let x = 0;
let y = 1;
```

In patterns, name bindings are prefixed with `$`. You can also destruct more complex patterns, if they're irrefutable.

```
let get_age(s: Student): Int = {
    let pattern Student { age: $age, .. } = s;

    age
};
```

`let pattern` can be used in anywhere, both in top-level `let` statements and scoped `let` statements.

## Functions

- Every function in Sodigy is pure.
  - Sodigy (and many other purely functional languages), doesn't consider terminating the entire program as an impure behavior. That's why `panic`, `assert` and many other debug functions are pure functions.
- Every function in Sodigy is evaluable at compile time.

### Generics

Sodigy's generic syntax is like that of Rust. Below, `id` is a generic function that returns itself.

```
let id<T>(x: T): T = x;

@test.true
let id_test1 = id(3) == 3;

@test.true
let id_test2 = id('a') == 'a';
```

You can either explicitly give the generic parameter, or let the compiler infer it. In the above example, the compiler infers `T`. To give the generic parameters, do it like below.

```
let id<T>(x: T): T = x;

@test.true
let id_test1 = id(Int, 3) == 3;

@test.true
let id_test2 = id(Char, 'a') == 'a';
```

If you give 2 parameters to `id`, the first one is a generic. The rule is like below.

Let's say a function `f` takes M generic parameters and N input parameters.

- If M + N parameters are given, the first M parameters are generic, and the last N parameters are input.
- If N parameters are given, it's input.
- Otherwise, it's an error.
  - You have to specify all the generics, or not at all.

You might think you can rely on compiler's type inference instead of generics. But you can't do that. Below doesn't compile.

```
let id(x) = x;

@test.true
let id_test1 = id(3) == 3;

@test.true
let id_test2 = id('a') == 'a';
```

Since `id` is not generic, `x` must have a single, concrete type. If `x` has type `Int`, then `id_test2` is wrong. If it's `Char`, vice versa.

### Lambda Functions

The syntax of lambda functions is very simple: parameters and the body are inside a curly brace, and the curly brace follows a backslash (`\`).

```
\{x: Int, y, x + y}
```

Above is an anonymous function that takes two integers and returns the sum of the integers. The last element inside the curly brace is the body of the lambda, and the others are its arguments.

Lambda functions can also capture its environment (closures).

```
let adder(n: Int): Func(Int, Int) = \{x: Int, x + n};

@test.eq(8)
let adder_test: Int = adder(5)(3);
```

## Values

### String Literals

In Sodigy, a String is a `List(Char)`. See [`Char`](#character).

Beside normal string literals, there are two special ones: formatted strings and bytes.

#### Formatted strings

A letter `f` followed by a string literal makes a formatted-string.

```
{
    let a = 3;
    let b = 4;

    f"\{a} + \{b} = \{a + b}"
}
```

The above value is evaluated to `"3 + 4 = 7"`.

#### Bytes

`Bytes` in Sodigy is a `List(Byte)`. Unlike [`Char`](#character), a `Byte` represents a byte in UTF-8 sequence.

```
@test.eq((3, 9))
let bytes = {
    let s = "가나다";
    let b = b"가나다";

    (s.len(), b.len())
};
```

### Character

A `Char` in Sodigy represents a code point. It's a very thin wrapper over an integer.

```
@test.true
let chars = {
    'a' as Int == 97
    && '가' as Int == 44032
};
```

### `if` expressions

`if` is an expression in Sodigy.

```
let x = if cond() { 3 } else { 4 };
```

It'd be very familiar if you know Rust/Haskell/Elixir, or any other functional language. If you're from C/C++, you must be familiar with ternary operators. That's an `if` expression.

#### `if pattern` expressions

It's like `if let` of the Rust language, but the keyword is different.

```
let foo: Option(Int) = Some(3);
let x = if pattern Some($x) = foo { x } else { 0 };
```

### `match` expressions

The syntax resembles that of Rust, except that it requires `$` before a name binding.

```
match foo() {
    Option.Some([$a, $b, $c, ..]) => $a + $b + 1,  # at least 3 elements
    Option.Some([$a, $b]) => $a + $b,  # exactly 2 elements
    Option.Some([]) => 0,
    Option.Some(_) => -1,  # matches any list
    Option.None => -2,
}
```

## Types

Types in Sodigy are first-class objects. The type checker (which is not implmeneted yet) evaluates the type signatures in compile time, and calls `.is_subtype_of()`.

### Integers

Sodigy uses arbitrary-width integers. There's no integer overflow in Sodigy.

### Ratio

Sodigy doesn't use floating points, but rational numbers. It gives you much more precise results, but is more expensive.

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

There's no named tuple in Sodigy. All the fields in structs must have a name.

### Tuple

Tuples are like that in C++/Rust and Python. You can pack values with different types in a tuple.

```
# it's how a type annotation of a tuple looks like
let a: Tuple(Int, Int, String) = (3, 4, "a");

# it's how you access an element in a tuple
@test.eq(3)
let b = a._0;
```

### List

`[1, 2, 3]`

## Operators

### `` ` ``

You can make an infix-operator using a backtick (`` ` ``). An operator is `` ` `` followed by an identifier without whitespace. The operator modifies a value of a field. The identifier is the name of the field that you want to modify. See how `` `age `` works below.

```
let struct Person = {
    age: Int,
    name: String,
};

# it modifies the `age` field of `p` and returns the modified version
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

@test.eq("Hello, World!")
let hello_world = "Hello, " <> "World!";
```

### `+>`

`+>` prepends an element to a list. But very unfortunately, it's not right-associative. Sodigy doesn't implement any kind of right-associativity.

```
@test.eq([0, 1, 2, 3])
let prepend_test = 0 +> [1, 2, 3];
```

### `<+`

`<+` appends an element to a list.

```
@test.eq([1, 2, 3, 4])
let append_test = [1] <+ 2 <+ 3 <+ 4;

@test.eq("Hello")
let append_test2 = "H" <+ 'e' <+ 'l' <+ 'l' <+ 'o';
```

### `..`

`..` makes an exclusive range. For example, `1..4` is a range: 1, 2, and 3, and `'a'..'c'` is `'a'` and `'b'`. An extra argument can set the step of the range. For example, `1..10..2` is `1`, `3`, `5`, `7`, and `9`. Negative steps are also possible.

You can index lists and strings with a range. For example, `a[0..3]` takes the first 3 elements of `a`. Or, `a[-3..]` takes the last 3 elements.

`..` operators are also used in patterns, but they mean a bit different in that case. See the examples below.

```
if pattern 0..3 = x {
  "x is 0, 1, or 2!"
} else {
  f"x is \{x}"
}
```

For numbers and characters, `..` in patterns are just range operators, like that in expressions. But you can also use `..` in string patterns. See below.

```
if pattern "ab".."cd" = x {
  "`x` starts with \"ab\" and ends with \"cd\"!"
} else {
  x
}
```

`"ab".."cd"` means a string that starts with `"ab"` and ends with `"bc"`. It's not a range operator, but a concat operator. You can also chain multiple strings, like below.

```
@test.true
let multiple_strings = if pattern "ab".."cd".."ef" = "aabbccddeeff" { True } else { False };
```

### `..~`

`..~` is like `..`, but includes the last index. For example, `1..~3` is `1`, `2` and `3`.

It's very useful in some cases. For example, if you want a pattern that covers lower case alphabets, it's either `'a'..'{'` or `'a'..~'z'`. The second one looks much better, doesn't it?

### `in`

`in` checks membership. It's like `.contains` in Rust, or `in` operator in Python.

### `as`

`as` operator casts types.

TODO: semantics of `as` in fallible and infallible cases. ex) `"3" as Int` and `"x" as Int`

### `?`

Sodigy uses `?` to handle errors. It's like `Maybe` monad of Haskell. It's kinda similar to `?`s in Rust, but not the same.

```
# Don't forget to add `?` after `n`.
# TODO: how about removing `?` in the function definition?
let foo(n?: Int): Option(Int) = if n == 0 {
  None
} else {
  Some(n - 1)
};
```

Let's say you want to call `foo` 3 times, like `foo(foo(foo(3)))`. Since the return type and input type of `foo` are different, you cannot call it like that. In this case you can use `?` operators like `foo(foo(foo(3)?)?)`. If any of intermediate result is `None`, the final result would be `None`. If it's `Some(Int)`, the calculation continues.

Below shows how you use `foo()?` in Rust.

```rust
fn foo(n: u32) -> Result<u32, ()> {
  if n == 0 {
    Err(())
  } else {
    Ok(n - 1)
  }
}

// This is how you use `?`s in Rust
foo(foo(foo(3)?)?)
```

The code is very similar, but they do different things. In Rust, `?` returns the current function if it's `Err`. But in Sodigy, there's no `return`. It only evaluates, and nothing returns.

Below is a Haskell version.

```haskell
foo :: Int -> Maybe Int
foo 0 = Nothing
foo n = Just $ n - 1

-- Equivalent code in Haskell
foo 3 >>= foo >>= foo
```

`foo 3 >>= foo >>= foo` in Haskell and `foo(foo(foo(3)?)?)` in Sodigy are almost identical, except that the Sodigy version is a bit more generic. You'll see the details a few paragraphs later.

One thing to notice about `?` is that the definition of `foo` tells that `n` is a `?`-able argument. In order to use `?` operators, you have to mark the function argument with a `?`. When an argument is `?`-able, it can `?`-ed types as an input.

For example, `n` in `let foo(n?: Int)` can be `3`, `None?`, `Some(3)?`, `Ok(3)?`, `Err(e)?`, ... and many other `?`-able types. The compiler generates multiple versions of `foo`: one without `?` and ones with `?`. When you call `foo` without any question mark, the type checker will choose the version without `?`, and nothing special happens. When you call `foo` with a `?`, the special version is chosen. The special version looks like below.

```
let foo_special<T>(n: Result(Int, T)): Option(Int) = match n {
  Ok($n) => foo(n),
  Err(_) => None,
};
```

The compiler first checks whether it can convert a `Result(Int, T)` into an `Option(Int)`. The answer is yes, and it makes a new function like above. `?`-conversion is defined by ___ type class (TODO: type classes). Sodigy std lib defines below conversions.

| From             | To                | Condition                           |
|------------------|-------------------|-------------------------------------|
| `Option(T)`      | `Option(T)`       | Always                              |
| `Result(T, E)`   | `Option(T)`       | Always                              |
| `Result(T, E1)`  | `Result(T, E2)`   | When `Into(E1, E2)` is implmeneted  |

- When a function has multiple `?`-ed arguments, the order of evaluation is undefined. It has to be specified, but I have to do more investigation.
- `?`-ed expression is not a special syntax. `?` is just a normal postfix operator. You can even make a list of `?`-ed integers, like `[Some(3)?, None?, Some(5)?]`. `Ok(3)?` and `Some(3)?` have different types, though.

### Logical operators

Sodigy has `&&` and `||`: 'logical and' and 'logical or'. Only `Bool` type implements `&&` and `||`, and you cannot implement your own versions for the other types.

### Bitwise operators

Sodigy has `^`, `&` and `|`. `>>` and `<<` are WIP. Sodigy's bitwise operation is a bit different from other languages. Integers in Sodigy have arbitrary lengths, like in Python. That means binary representation of `7` in Sodigy has three `1`s and infinite number of leading `0`s. Due to this, there's no `~` in Sodigy. `~` on any positive number will result in infinite (not sure whether that's positive or not).

That also makes bitwise operations on negative numbers very complicated. There's no 2's complement or 1's complement in Sodigy. It internally uses a sign bit, but we cannot apply bitwise operations on that. So it raises a runtime error when you try to do bitwise operations on negative numbers. This might change in the future.

There are a few other useful functions. They are only available for positive numbers.

- `leading_ones`
  - `(0b1110101).leading_ones() == 3`
- `trailing_ones`
  - `(0b1010111).trailing_ones() == 3`
- `trailing_zeros`
  - `(0b1101000).trailing_zeros() == 3`
- `count_ones`
  - `(0b1101011).count_ones() == 5`
- `ilog2`
  - `(0b1101011).ilog2() == 7`
  - It counts the bit width without the leading zeros.

#### Shift

You can shift negative counts. For example, `a << -5` is `a >> 5`.

## Comments

`#` for a single line comment, `#>` for a doc-comment and `#!` \~ `!#` for a multiline comment.

```
# This is a comment
let add_1_2: Int = 1 + 2;

#!
This is also a comment
!#

#> This function adds two numbers.
let add(x: Int, y: Int): Int = x + y;
```

`#! ... !#`s can be nested, of course.

## Macros

`@[MACRO_NAME](x, y, z)`.

There are no specs for macros, only rough sketches for them.

1. `MACRO_NAME` is a single identifier that identifies the macro. The list of macros are at `sodigy.toml`. It tells you where to find the definitions of the macros.
2. Macros are implemented in Sodigy. It's like how procedural macros work in Rust. A macro is a sodigy function that takes `List(Token)` and returns `Result(List(Token), CompileError)`.
3. The compiler uses its interpreter to run macros. That means macros make the compilation VERY SLOW.

## Modules and Imports

There are two keywords in Sodigy for modules: `module` and `import`.

`module foo;` defines a module named `foo`. The code must be at `./foo.sdg` or `./foo/lib.sdg`. `module foo;` implies `import foo;`. Your code can use names in `foo` with dots. For example, if `bar` is defined in `module foo`, it can be accessed with `foo.bar`.

TODO: we need better specifications for `module`s and `import`s. for now, there seems to be no reason for `module` to exist!

The `import` keyword imports external modules. There are 4 ways you can link external Sodigy files. Let's say you did `import foo;`

1. You can tell the compiler where `foo.sdg` is. TODO: it's not implemented yet
2. Local files. For `import foo;`, the compiler first looks for `./foo.sdg` and `./foo/lib.sdg`. If either of them exists, the compiler links the file.
3. You can specify the path of the `.sdg` file in `sodigy.toml`. The `sodigy.toml` file must be at `.`.
4. Standard Library. TODO: not implemented yet

The compiler tries in that order.

TODO: I want the compiler to warn unused dependencies. For ex, if there's `foo = { path = "../foo.sdg" }` in `sodgiy.toml` but no one imports `foo`, the compiler should warn that!

You cannot `import *;` like many other languages. Sodigy cannot detect/prevent cyclic imports.
