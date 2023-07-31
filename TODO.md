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
  Some($n) { foofoo(n) }
  None { None }
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
  T1($n) { foofoo(n) }
  T2($c) { R2(c as B) }
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
      T1($val2) { foo(val2) }
      T2($c) { R2(c as B) }
    }
  ]
}
```

isn't it monad?

For this, we have to clarify the order of evaluation of args

`foo(a?, b?, c?)`: what if all the args are erroneous?

---

Pattern matching

- `a = foo();`
  - `$a = foo();` should be the one, but `a = foo();` is a syntactic sugar
  - this syntactic sugar is only allowed in `let`s in a block expression. it's not allowed in match statements
- `a: Foo = foo();`
- `Foo { name, age } = foo();`
  - `_tmp: Foo = foo(); name = _tmp.name; age = _tmp.age;`
- `mod.Foo { name, age } = foo();`
- `Foo { name: $x, .. } = foo();`
- `Ok($n) = try_something();`
  - only in `match`s
- `Bool.True = try_something();`
  - only in `match`s
- `[$a, $b] = foo();`
- `[$a, $b]: List(Int) = foo();`
  - what if `foo().len()` is greater than 2? do we reject that?
- `[$a, ..] = foo();`
- `[$a, .., $b, $c] = foo();`
  - what if `foo().len() == 2`? do we reject that? if `a` and `b` point to the same object, that's non-sense
- `($a, $b) = foo();`
- `($a, _) = foo();`
  - we should treat `_` specially: it's not allowed as an identifier, except inside a pattern
  - If inside pattern, it's ignored (no bindings)
- `[Ok(Foo { name, age }), ..] = foo();`
  - only in `match`s

---

syntax sugar for the below pattern

```rust
if let PATTERN = VALUE { true } else { false }
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
