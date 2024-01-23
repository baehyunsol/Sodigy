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

Type Classes


https://smallcultfollowing.com/babysteps//blog/2016/09/24/intersection-impls/
https://smallcultfollowing.com/babysteps//blog/2016/09/29/distinguishing-reuse-from-override/
https://smallcultfollowing.com/babysteps/blog/2016/10/24/supporting-blanket-impls-in-specialization/
https://smallcultfollowing.com/babysteps/blog/2022/04/17/coherence-and-crate-level-where-clauses/
https://aturon.github.io/tech/2017/02/06/specialization-and-coherence/
https://github.com/purescript/documentation/blob/master/language/Type-Classes.md

Syntax/Semantic 생각해보기!

`Add for (Int, Int)` 이런 식으로 추가하잖아? custom compiler error도 던질 수 있게 하자! `Add for (List(Any), List(Any))` 하면 무슨 compiler error 던질 지를 Sodigy로도 정할 수 있게 하기!

---

Rust does not allow `if Point { x, y } == p { .. }` -> curly braces in conditions of if branches. They force us to use parenthesis around `Point { .. }`.

1. See how they implement such checks

---

Generics with restrictions

`let add<T: Add(T, U, T), U: Add(T, U, T)>(a: T, b: U): T = a + b;`

- we need some kinda restrictions inside `<>`. What if the type parameters have `>` or `<` within it?
- how about `let add {T: Add(T, U, T), U: Add(T, U, T)}(a: T, b: U): T = a + b;`?
  1. 대부분 언어에서 `<>` 쓰기 때문에 헷갈릴 수도 있음!
- 아니면, 저런 조건 하나도 쓰지 말고 걍 컴파일러가 알아서 찾으라고 하기
  1. `let add<T, U>(a: T, b: U): T = a + b;`라고만 하면 나중에 얘가 type solving 하면서 `Add(T, U, T)`를 알아서 찾을 거 아녀. 그때 에러를 날리든가 말든가 하는 거지
  2. 이러면 깔끔한 에러를 날릴 수가 있나?? 나중에 가서 누가 `T = String`, `U = Bool`로 instantiate 하려고 하면 `Add(String, Bool, String)`을 찾을 수 없다고 에러 날릴텐데 그럼 안 헷갈리나?
    - 최소한 `+`의 span을 기억해뒀다가, 그거 밑줄 정도는 쳐 줘야함

---

Generic function의 type checking은 언제 하는 거임??

- `let id<T>(x: T): T = x;`
- `let always_error<T>(x: T) = id(x) + (3 + "asdf");`
- `let sometimes_error<T>(x: T) = id(x) + 3;`

만약에 아주 다양한 type을 이용해서 저 함수들을 initialize 한다고 치자...

1. `always_error`는 매번 새로운 type error를 던지나? 그냥 `always_error` 본문만 읽고 type error 한번만 던지면 안되나?
2. trait system을 사용하면 `sometimes_error`는 에러가 한번만 나거나 0번 나거나 둘 중에 하나임. `sometimes_error(True)`하고 `sometimes_error("")`하고 함수 호출에서 에러가 나지 함수 정의에서는 에러가 나지 않음. (정의에서 에러가 났으면 호출에선 에러가 안 났을 거고)
