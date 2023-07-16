# Sodigy

Sodigy is a very abstracted, purely functional programming language.

It's still under development. Only parser and lexer are (partially) complete.

## Functions

- Every function in Sodigy is pure.
- Every function in Sodigy is evaluable at compile time.

### Lambda Functions

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

```
{
    x = 3;
    y = 4;
    z = {
        x = 5;
        w = 6;

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
    x = 3;
    y = 4;
    x = 5;

    # Don't know which `x` to use
    x + y
}
```

```
{
    x = z + 1;
    y = x + 2;
    z = y + 3;

    x + y + z
}
```

## Types

Types in Sodigy are first-class objects. The type checker (which is not implmeneted yet) evaluates the type signatures in compile time, and calls `.is_subtype_of()`.

### Integers

Sodigy uses arbitrary-width integers.

### Numbers

Sodigy doesn't use floating points, but rational numbers.

## Operators

### `$`

`$` is an infix operator, which modifies a value of a field.

Its lhs operand is the object you want to modify. Unlike the other infix operators, it has 2 rhs operands: the name of the field and the new value.

```
# TODO: add definition of `Person`

def set_age(p: Person, new_age: Int): Person = p $age new_age;

@test.eq(Person("Bae", 23))
def set_age_test: Person = set_age(
    Person("Bae", 21), 23
);
```

You can use whitespaces between `$` and the name of the field, but I recommend you not to do so, for the sake of readability.