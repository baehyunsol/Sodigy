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

publicity

`@public`: 100% public (default)

`@private`: within this module (this file)

`@public.submodule`: within this module and its submodules

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
2. slow compilation: interpreted Sodigy is much slower than the compiled one

whats different from Rust

- let's make error messages show tokens generated from macros
  - Rust error messages don't show the generated tokens, it only shows the macro invocation
  - let's show both the invocation and the results

---

`import * from x;`

- hinders incremental compilations
- 아니 사실 불가능함.
  - x가 `import * from y;`하고 y가 `import * from x;`하면 어떡함? 현재로써는 저거 해결할 방법이 없음. 저거 해결하려면, name collect와 name resolve를 별개의 IR 단계에서 처리하고, 그 사이에서 저 `*`을 처리해야함...
  - `*` 하나때문에 IR 단계 추가하는 거는 별로...

---

IRs later

Mid-IR: every function (including imported ones) is converted to Uid. No more identifiers. All the operators are also lowered to func calls, which use Uids. Everything has a type.

Low-IR: everything is either array or integer. For example, a rational number is an array of length 2 (2 integers). A struct is an array whose elements are its fields. Field access operator is just an array indexing operator.

---

Compile time evaluation

1. a sodigy function `comptime(v)` guarantees that `v` is evaluated at compile time
2. a decorator `@comptime` guarantees that the function it decorates is called at compile time
3. an annotated block `comptime { .. }` guarantees that the code inside the block is evaluated at compile time
  - ugly

seems like the second one is the least ugly one

---

`let`대신 `def` 쓸까?
`if let`대신 `def if` 쓰고
