- Parsers of block_expr and lambda_def rely on the fact that commas and semi-colons do not appear in expressions. They only appear inside `{}`s, `[]`s or `()`s. -> How do I guarantee that using code?

- incremental compilation
  - reuse intermediate result from previous compilations
- background compilation
  - daemon iterates all the files regularly
    - period: 5 seconds -> very long for CPU, but short enough for programmers
  - if it finds a modified file, it tries to generate an intermediate result
    - if succeeds, update the intermediate result
    - if fails, let programmers use information from the error messages

---

```
def foo(n: Int?): Option(Int) = foofoo(n);
```

becomes

```
# syntax is WIP

def foo(n: Option(Int)): Option(Int) = match n {
  Some(n) => foofoo(n),
  None => None,
};
```

more generalization

```
def foo(n: A?): R(X, B) = foofoo(n);
```

becomes

```
# syntax is WIP

def foo(n: T(A, C)): R(X, B) = match n {
  T1(n) => foofoo(n),
  T2(c) => R2(c as B),
};
```

we want `n` to be anytype that implements `?`, not just `T`. We also want `R` to implement `?`, because we have to choose either `T2(c) => R1(c as X)` or `T2(c) => T2(c as B)`.

callers have to be explicit about `?`.

```
{
  val1: A = _;
  val2: T(A, C) = _;

  [
    foo(val1),
    foo(val2?),
  ]
}
```

isn't it monad?
