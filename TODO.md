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

```
@test.before(\{assert(x > 0 && y > 0)})
def foo(x: Int, y: Int) = x + y;

@test.after(\{ret, assert(ret >= 0)})
def sqr(x: Int): Int = x * x;
```

`test.before` is called before the actual function is called.

`assert` is an action (not a function) that works like rust's `assert!`.

functor of `test.after` takes one input: the return value of the function its decorating

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
def foo(x?: X): Y = bar(x);

# becomes

def foo_real(x: X): Y = bar(x);

# each `T` creates new one
def foo_quest(x: T(X, Y)): Y = match x {
  T.T1($x) => bar(x),
  T.T2($err) => ##! there must be a variant of Y for this case !## .. ,
};
```

---

Macros

the best (and the only) way I can think of

like that of `[proc_macro]` in Rust:

a Sodigy function that takes `List(TokenTree)` and returns `List(TokenTree)`

the compiler, which is written in Rust is going to impl interpreter: it can run the macro

so, the step would be

1. Compiler(Rust): Code(Sodigy) -> Vec<TokenTree>
2. Compiler(Rust): Vec<TokenTree> -> List(TokenTree)
3. Macro(Sodigy): List(TokenTree) -> List(TokenTree)
4. Compiler(Rust): List(TokenTree) -> Vec<TokenTree>
5. Compiler(Rust): continue...

the macro should be compiled independently

limits

1. incremental compilation: when macro is modified, but the code isn't
2. slow compilation: 
