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
let foo(x: Int, y: Int) = x + y;

@test.after(\{ret, assert(ret >= 0)})
let sqr(x: Int): Int = x * x;
```

`test.before` is called before the actual function is called.

`assert` is an action (not a function) that works like rust's `assert!`.

functor of `test.after` takes one input: the return value of the function its decorating

---

```
let foo(x?: Int, y?: String): Result(Int, Err) = Result.Ok(bar(x, y));
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
let foo(x?: X): Y = bar(x);

# becomes

let foo_real(x: X): Y = bar(x);

# each `T` creates new one
let foo_quest(x: T(X, Y)): Y = match x {
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

Like that of `[proc_macro]` in Rust. There's a sodigy function that takes `List(TokenTree)` and returns `List(TokenTree)`. Below is the compilation step.

1. Compiler(Rust): Sodigy Code -> Vec<TokenTree>
2. Compiler(Rust): Vec<TokenTree> -> List(TokenTree)
3. Macro(Sodigy): List(TokenTree) -> List(TokenTree)
4. Compiler(Rust): List(TokenTree) -> Vec<TokenTree>
5. if there're remaining macros, goes back to step 2 

Macros should be compiled independently

std macros

- `@[max](a, b)` -> `if a > b { a } else { b }`
  - can take an arbitrary number of arguments (at least 1)
  - using functions must be much better way to do this...
    - `let max2(x, y) = if x > y { x } else { y }`
    - `let max3(x, y, z) = if x > y { if y > z || x > z { x } else { z } } else if y > z { y } else { z }`
- `@[min](a, b)`
  - see `max`
- `@[map](x: y, z: w, 0: 1)`
  - like that of Python
- `@[set](x, y, z)`
  - like that of Python
- `@[generate](iterate 3..10; filter x % 2 == 0; map x * x;)`
  - list comprehension

how does one import a macro? the compiler knows the imported names at the hir level, while the macros are needed at TokenTree level. there must be some other syntax for importing macros. for now, the only way I can think of is using another file for metadata, like `Cargo.toml` or `go.mod`

name issues with `@[map]`: how does it know the name of std.hash_map? what if the preluded name is overloaded?
- how about protecting absolute paths? so that the full name of `Map` never changes, ex: `Sodigy.prelude.Map`, in this case, a new definition of `Sodigy` would be rejected by the compiler

---

`import * from x;`

impossible: due to cyclic imports

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

more feature rich f-strings

- integer
  - `:x`: lowercase hex
  - `:X`: uppercase hex
  - `:o`, `:O`: oct
  - `:b`, `:B`: bin
  - `:#x`, `:#X`, `:#o`, `:#O`, ... : prefix `0x`, `0o` or `0b`
- stretch, align, fill
  - make the output string length s, align the string left/right/center, and fill the empty space with c
- rational numbers
