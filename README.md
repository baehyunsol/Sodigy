# Sodigy

Sodigy is a very abstracted, purely functional programming language.

It's still under development. Only parser and lexer are (partially) complete.

## Functions

- Every function in Sodigy is pure.
- Every function in Sodigy is evaluable at compile time.

### Lambda Functions

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