- lex
  - code -> token
- parse tree
  - token -> token tree
- parse
  - token tree -> AST

---

spec

기존 Sodigy의 스펙 최대한 따라가기!!

추가사항

`##>`: Doc comment (how about `##!>` for multi-line doc comments?)

pattern guard -> rust와 동일, match 안에서만 사용 가능 (let에서는 사용 불가능)

tuple에서 field 접근 -> `.0`, `.1` 말고 `._0`, `._1` 하자...

generic type annotation: `Some(3)`, `Some(Int, 3)`
  - distinguish by the number of arguments

---

rustc: termcolor라는 crate 쓰는구만...

---

`let x = 3;` vs `let $x = 3;`

1. `($x, $y)`에서는 `$` 붙이고 `x`에서는 안 붙이면 헷갈린다.
2. 정의는 `$x`로 하고 쓸 때는 `x`로 쓰면 헷갈린다.

---

```
def foo(x?: Int, y?: String): Result(Int, Err) = Result.Ok(bar(x, y));
```

```
foo(val1?, val2?)

# becomes

match val1 {
  T.T1($n) => match val2 {
    V.V1($s) => Result.Ok(bar(n, s)),
    V.V2($err) => Result.Err(err as Err),
  },
  T.T2($err) => Result.Err(err as Err),
}
```

1. `?` is an OPERATOR: `x?` changes the value and type of `x`.
2. If `x`'s type is `T(X, Y)`, then `x?` has type `Question(T(X, Y))`.
3. If `foo(x?: X)` exists, then `foo`'s result changes according to the type of the first arg (which is solid)
  - that would be like below

```
def foo(x:? X): Y = bar(x);

# becomes

def foo_real(x: X): Y = bar(x);

# each `T` creates new one
def foo_quest(x: T(X, Y)): Y = match x {
  T.T1($x) => bar(x),
  T.T2($err) => ##! there must be a variant of Y for this case !## .. ,
};
```
