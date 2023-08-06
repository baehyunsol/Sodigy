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
  Option.Some($n) => foofoo(n),
  Option.None => Option.None,
};
```

more generalization

```
def foo(n?: A): R(X, B) = foofoo(n);
```

becomes

```
# syntax is WIP

def foo(n: T(A, C)): R(X, B) = match n {
  T1($n) => foofoo(n),
  T2($c) => R2(c as B),
};
```

we want `n` to be anytype that implements `?`, not just `T`. We also want `R` to implement `?`, because we have to choose either `T2(c) => R1(c as X)` or `T2(c) => T2(c as B)`.

callers have to be explicit about `?`.

```rust
{
  let val1: A = _;
  let val2: T(A, C) = _;

  [
    foo(val1),
    foo(val2?),
  ]
}

# becomes

{
  let val1: A = _;
  let val2: T(A, C) = _;

  [
    foo(val1),
    match val2 {
      T1($val2) => foo(val2),
      T2($c) => R2(c as B),
    }
  ]
}
```

isn't it monad?

For this, we have to clarify the order of evaluation of args

`foo(a?, b?, c?)`: what if all the args are erroneous?

---

syntax sugar for the below pattern

```rust
match val {
  PATTERN => true,
  _ => false,
}
```

---

more testing

How do I state this: ``` for all ls: List(Int), `ls.sum() == ls.sort().sum() && ls.len() == ls.sort().len() && ls.sort().is_sorted()` ```

I want that to be attached to `List.sort`

---

rand functions

- pure one
  - `State { curr: Int }`
  - `.init(seed: Int): State = State { curr: seed };`
  - `.next(self): (State, Int) = (self $curr hash(self.curr), self.curr);`
- impure one
  - `.rand_int(): Int`
  - use it to initialize a pure one
