- Parsers of block_expr and lambda_def rely on the fact that commas and semi-colons do not appear in expressions. They only appear inside `{}`s, `[]`s or `()`s. -> How do I guarantee that using code?

- Make multiple crates
  - current crate only parses a file
    - doesn't care about other files
    - returns `Vec<Stmt>`
      - TODO: return `AST`, which does more analysis on `Vec<Stmt>`
        - it has
          - all the names (`def`ined or `use`d)
          - very basic optimizations
            - `{x = foo(); y = bar(); x + y + y}` -> `{y = bar(); foo() + y + y}`
            - `{x = \{x, x + 1}; x(a)}` -> `{__TMP_LAMBDA_FUNC_NAME(a)}` -> `__TMP_LAMBDA_FUNC_NAME(a)`
    - it also does name-resolving
    - it also does name-checking
      - If an expression has a symbol `a`, `a` must be
        - defined in the file,
        - imported with `use __ as a;`,
        - or in `sodigy.prelude`
        - none of them have to do with other files
      - TODO: how about name errors with `use`s?
        - ex: `use a.b.c;` where `a` is an external file
          - `b` is not defined in `a`
          - do we have to preserve span of `b` for error messages?

- incremental compilation
  - reuse intermediate result from previous compilations
- background compilation
  - daemon iterates all the files regularly
    - period: 5 seconds -> very long for CPU, but short enough for programmers
  - if it finds a modified file, it tries to generate an intermediate result
    - if succeeds, update the intermediate result
    - if fails, let programmers use information from the error messages

---

`b"ABC"` -> `bytes([65, 66, 67])`

`f"{a} + {b} = {a + b}"` -> `a.to_string() <> " + " <> b.to_string() <> " = " <> (a + b).to_string()`

In order to implement these, there must be a way the compiler can represent builtin functions (in AST)

- `bytes`
- `.to_string()`

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