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

`?` syntax

```
def foo(a: BAR, b: BAZ): Result(Ty) = func(
  foofoo(a)?, barbar(b)?
);
```

TODO: study Monad

- `f(state, val) -> (new_state, result)`
  - don't want to destrut every time
- `Option::map()`
  - can implement with typical syntax
- `?`
  - needs a special syntax and semantics, but seems difficult to impl

---

```
use a.b.c;

{a = 3; a + c}
```

after name resolving, `a + c` would become `a + a.b.c`, and `a` in rhs would be 3. -> It's not what the programmer intended...