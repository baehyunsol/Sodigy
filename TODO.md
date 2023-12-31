spec

기존 Sodigy의 스펙 최대한 따라가기!!

추가사항

pattern guard -> rust와 동일, match 안에서만 사용 가능 (let에서는 사용 불가능)

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

Macros

Like that of `[proc_macro]` in Rust. There's a sodigy function that takes `List(TokenTree)` and returns `List(TokenTree)`. Below is the compilation step.

1. Compiler(Rust): Sodigy Code -> Vec<TokenTree>
2. Compiler(Rust): Vec<TokenTree> -> List(TokenTree)
3. Macro(Sodigy): List(TokenTree) -> Result(List(TokenTree), CompileError)
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

can macros nested?

1. compiler expands macro over and over until no macro is found
2. The one who implements `Func(List(TokenTree), Result(List(TokenTree), CompileError))` must tell the compiler whether the result has another macro or not

both make sense

---

`import * from x;`

impossible: due to cyclic imports

in order to resolve `import * from x;`, one has to collect all the names in `x`. the collecting and name resolving is done at the same time. that means if there are more than two modules `import *`ing each other, the compiler cannot do anything

---

IRs later

Mid-IR: every function (including imported ones) is converted to Uid. No more identifiers. All the operators are also lowered to func calls, which use Uids. Everything has a type.

Low-IR: everything is either list or integer. For example, a rational number is a list of length 2 (2 integers). A struct is a list whose elements are its fields. Field access operator is just a list indexing operator.

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

---

Python operator

- `**`
  - power
  - has no problem with syntax
  - but we already have `a.pow(b)`
- `//`
  - integer division
  - but we distinguish `3 / 4` and `3.0 / 4.0`. how about for `a: Int` and `b: Int` -> `a / b` vs `a.into(Ratio) / b.into(Ratio)`
    - very verbose
    - how about `a as Ratio / b as Ratio`
- `in`
  - `a in b` is way more straighforward than `b.contains(a)`
- `as` (in Rust)
  - it's already a keyword!
  - `a as Ratio` is more straightforward than `a.into(Ratio)`

`in`, `as`, `**`는 추가하고 `//`는 추가하지 말자

- `2 ** 3.0`
  - 당연히 exp가 Int일 때랑 Ratio일 때랑 구현이 달라야 함. 당연히 Int일 때가 더 효율적이겠지. 근데 `2 ** 3.0` 보고 compiler가 최적화 때려도 됨? 그럼 `2 ** a == 2 ** 3.0`이 `False`가 될 수도 있음 (`a == 3.0`일 때)...
  - 그렇다고 구현을 다르게 해버리면 `2 ** 3.0 == 2 ** 3`이 `False`가 됨

---

bitwise operations

- `&`
  - already exists
- `|`
  - already exists
- `^`
  - already exists
- `~`
  - impossible in inf-width int
- `<<`
  - already exists
- `>>`
  - already exists
- count_ones
- ilog2
- trailing_ones
- trailing_zeros
- leading_ones
  - impossible in inf-width int
- leading_zeros
  - impossible in inf-width int

how do they deal with negative numbers? for now, it doesn't use 2s complement... what's binary representation of `-1`? infinite 1s?

how about `first_n_bits(n: Int)`, `last_n_bits(n: Int)`

---

documents

compiler outputs data for documentation (maybe in JSON?)

it contains...

- types
- doc comments
- uid
  - it doesn't help readers, but it would make it much easier to implement document renderers
- dependency
  - who is calling this function
  - whom this function is calling

more fancy stuffs...

- example code is actually run in tests
  1. sodigy's test runner reads doc-comments and tries to run codes in the document
  2. special annotation includes a code snippet to the document

2 looks much better

```
def adder(n: Int): Func(Int, Int) = \{x, x + n};

# documentation of `adder` shows this example
@document.example(adder)
@test.eq(3)
def adder_ex = {
    let adder1 = adder(1);

    adder1(2)
};
```

---

types

MIR에서 모든 함수/operator의 uid를 찾아서 걔를 직접 때려박잖아? 근데 generic은 일단 유보하자. 즉, `a + b`가 있으면 `a`와 `b`의 type을 둘다 찾아서 `+`가 무슨 `+`인지 찾아서 걔의 uid를 넣는게 [[giant]]아니고[[/giant]], generic add의 uid를 넣어 놓는 거임. 나중에 type inference랑 type check가 끝나면 그때 real uid를 넣는 거지..

이제 구현이 쉬움, Operator랑 함수 호출이 똑같이 생겼거든!!

`foo(a, b, c)`를 볼 경우:

1. type check를 `a`, `b`, `c`에 recursive하게 호출, 걔네의 type을 전부 알아옴.
  - 다른 annotation으로부터 쟤네의 type을 알 수 있을 경우 걔네를 바로 사용.
2. a, b, c의 type이 foo의 input type과 일치하는지 확인
  - 만약 foo의 type이 불완전하게 제공되었으면??
3. 일치할 경우 `foo(a, b, c)`의 type을 알아낸 거임!
4. 만약 누군가 `foo(a, b, c)`의 type을 infer하고 싶었으면 걔한테 알려주면 됨. 만약 `foo(a, b, c)`에 type annotation이 붙어있었으면 걔가 정확한지도 확인해야함

---

REPL

- `let x = 3 + 4` 할 필요없이 `3 + 4`만 하면 됨
  - 일단 input을 받아서, `let`으로 시작하면 그대로 쓰고,
  - `let`이 없으면 `let main = `을 붙이자
- compile error는 다 보여줘야지, warning도 보여줘야 되나??

---

default values for struct

```
let struct Person = {
  age: Int = 32,
  name: String = "Bae",
};
```

... really?

---

function overloading with types

```
let into<T, U>(x: T): U = panic();  # not implemented

let into(x: Int): Ratio = Ratio.from_denom_and_numer(1, x);
```

- when it sees `"123".into(Int)`, it first looks for `into(x: String): Int`. if it cannot find one, it calls the default one
- the current implementation doesn't allow that: name collisions
- what if subtype-related stuff complicates the problem?

---

list implementation

- `Rust::Vec` way
  - `x[1..]` is O(n)
- Linked List
  - `x[n]` is O(n)
- `Rust::Slice` way
  - every `List` consists of a buffer, start index and end index
  - `x[n]`: O(1) -> buffer + start + `n`
  - `x[n..]`: O(1) -> start += n
  - `x.len()`: O(1) -> end - start
  - `x.modify(n, v)`: O(n) -> price for immutability
  - `a +> x`, `x <+ a`: O(n) -> it's fine

---

decorators (impl)

the current implementation is too messy. `FuncDeco`, `ArgDeco`, `EnumDeco`... an enum for every kind of deco? no...

I'm too obsessed with the idea that HIR has to handle decorators.

Implementing decorators in Sodigy? `@method(Int)`, `@public`, and much more... have to be built-in.

If i'm to implement them in Rust, do I have to hard-code all the symbols in the compiler?

1. use universal decorator type: `Hir::Decorator`
  - `{ DottedNames, Option<Vec<Hir::Expr>> }`
  - every `Ast::Decorator` is lowered to `Hir::Decorator`
2. Some obvious decorators are handled in Hir level
  - eg) `@public`, `@private`, `@test.eq`...
  - Hir doesn't do error handling at all! for ex, Hir cannot handle `@test.eqq(3)` because there's a typo. then it just lowers the decorator to `Hir::Decorator`, all the error handlings are done later
  - if `ast::Expr` to `hir::Expr` lowering fails, then HIR can handle that!

---

decorators (spec)

- test-related
  - `@test.eq(val)`: assert that the return value of the function it decorates matches the given value
  - `@test.true`, `@test.false`: `@test.eq(True)`, `@test.eq(False)`
  - `@test.expect(args, val)`: assert that `f(args) == val`
    - `@test.eq(val)` can be lowered to `@test.expect((), val)`
  - `@test.before(\{assert(x < 0)})`: when `f` is called, it always make sure that `x`, which is the input of `f`, is less than 0.
  - `@test.after(\{ret, assert(ret > 0)})`: when `f` is called, it always make sure that the returned value is greater than 0.
  - when are `@test.before` and `@test.after` enabled? only on test-mode? on test-mode and debug mode? or always?
- visibility
  - `@public`, `@private`
  - which one is the default?

---

linear type system (check in MIR)

- `let foo(x: Int, y: Int): Int = bar(...);`
  - check how many times `x` is used
    - none: warning
    - exactly once: maybe useful later when doing RC optimization
    - not known at compile time due to branch
  - check how many times foreign uid is used
    - for example `bar` is used...
    - at least once / 0 ~ more: use this info when building dependency graphs
      - for ex, if `foo` calls itself at least once, that's an infinite recursion
- `{let x = ...; let y = ...; ...}`
  - check how many times `x` is used
    - none: warning
    - exactly once: remove `let x = ...;` and replace `x` with its value
    - at least once: no need for lazy evaluation, just evaluate this eagerly when the scope is entered

---

How other languages import modules

- Python: `import A`
  - There must be `A.py` somewhere. There are predefined rules to find `A.py`.
- Rust: `use A`
  - `Cargo.toml` tells you what `A` is.
- Go: `import "A"`
  - `go.mod` and `go get A` tells you what and where `A` is.

---

Ratio: `denom.len()`이나 `numer.len()`이 64보다 커지면 줄이자

- 둘다 적당히 크면 LSB 날리고
- 분모만 크면 0이라고 해버리고
- 분자만 크면...
  - runtime error??
  - `inf`?? -> 이거는 어떻게 표현?
    - `1/0`하고 `-1/0`으로 표현할까? 이러면 if문 몇개 더 필요하긴한데 꽤 깔끔할 듯?

---

Conditional Compilation & Compile Time Function Evaluation

- `cond_comp(is_debug_mode(), print(x), x)`
  - `cond_comp`의 cond는 무조건 comp time에 evaluate 돼야 하고, 선택된 branch만 남음.
  - expression에만 사용가능
- `comp_time(x.type).variants`
  - type 관련된 친구들은 당연히 compile time에 전부 계산돼야지!
  - module 같은 친구들도 마찬가지고

---

How about this...

`match`에서 string pattern 쓸 때,

`"ab"..`나 `.."ab"`로 `starts_with`, `ends_with` 나타내는 거임! `"ab".."cd"`로 동시에 쓰는 것도 가능

how about `"ab"..$a.."cd"`? 이거 괜찮은데?? 점점 `p"(%d, %d)"`스러워지긴 하지만, 새로운 문법을 추가하는 건 아니니까 그나마 나은 듯?

---

Type Classes


https://smallcultfollowing.com/babysteps//blog/2016/09/24/intersection-impls/
https://smallcultfollowing.com/babysteps//blog/2016/09/29/distinguishing-reuse-from-override/
https://smallcultfollowing.com/babysteps/blog/2016/10/24/supporting-blanket-impls-in-specialization/
https://smallcultfollowing.com/babysteps/blog/2022/04/17/coherence-and-crate-level-where-clauses/
https://aturon.github.io/tech/2017/02/06/specialization-and-coherence/
https://github.com/purescript/documentation/blob/master/language/Type-Classes.md

Syntax/Semantic 생각해보기!

`Add for (Int, Int)` 이런 식으로 추가하잖아? custom compiler error도 던질 수 있게 하자! `Add for (List(Any), List(Any))` 하면 무슨 compiler error 던질 지를 Sodigy로도 정할 수 있게 하기!
