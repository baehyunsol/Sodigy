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
- `@[table](x: y, z: w, 0: 1)`
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

in order to resolve `import * from x;`, one has to collect all the names in `x`. the collecting and name resolving are done at the same time. that means if there are more than two modules `import *`ing each other, the compiler cannot do anything

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

clap 관련 참신한 아이디어

지금은 `Vec<String>`으로 된 args를 concat한 다음에 바로 parser에 넣잖아? 근데 이걸 한 스텝 더 하는 거임!

1. `Vec<String>`으로 된 args를 concat해서 span을 구함
2. 방금 만든 string을 Sodigy Code로 바꾸는데 span은 보존함. 예를 들어서,
  - `sodigy a.out --dump-hir true`를 `sodigy.input("a.out").dump_hir(true)`로 바꾸는 대신에 span은 cli input의 모양을 유지하는 거지! 그럼 에러메시지가 아주 예쁘게 나옴
  - 저런 식으로 변환하는 거는 아주아주 쉬움
    - flag인지 아닌지 구분하는 거는 trivial
      - flag를 보고 이게 valid한지 아닌지 판단하는 코드는 남겨둬야함 ㅋㅋㅋ 그건 sodigy가 못함. 사실 sodigy가 할 수는 있는데 너무 비효율적!
    - flag 뒤에 flag가 오면 앞의 flag의 input으로 걔의 default value를 주면 됨 (input을 안 받는 flag면 비워두면 되고)
    - flag 뒤에 non-flag가 오면 앞 flag의 input으로 해석
    - flag가 안 왔는데 input이 오면 걔는 input file.
      - 만약 사용자가 `--dump-hir a.out`라고 쓰고, `--dump-hir true a.out`을 기대했다면? 그건 어쩔 수 없음... 사용자 잘못 ㅋㅋㅋ
    - 이러면 좀 더 유연하게 value 해석 가능: `--dump-hir=true`, `--dump-hir true` 등등 전부 가능
3. 2번에서 나온 Sodigy Code를 interpreter로 돌려버리는 거임...
  - 여기서 또다른 아이디어 -> Python은 런타임 오류도 span 보여주잖아? Sodigy도 (optionally) 그게 되게 하면 여기서 더 예쁜 에러메시지를 뽑아낼 수 있을 듯?
  - 이런 건 어떰? debug mode에서는 span을 싹 다 보존하고, release에서는 싹 다 날리는 거임!
  - 사실 싹 다 보존한다고 쳐도 별로 안 비싼게, 오류가 날 수 있는 경우에만 보존을 하잖아? 근데 오류가 날 수 있는 경우가 그렇게 많지가 않음...
  - 오류 날리는 함수에 옵션으로 span 보존할지 말지 줄 수 있게 할까?? Python스럽고 좋기는한데 purity를 해칠 가능성이 농후함...

예시: `sodigy a.out --dump-hir true`를 아래처럼 해석

```
# op precedence가 저게 맞나?? ㅋㅋㅋ 기억이 안 나네..
@method(SodigyCompiler)
let input(self: SodigyCompiler, file: String): SodigyCompiler = self `input self.input <+ file;

@method(SodigyCompiler)
let dump_hir(self: SodigyCompiler, dump_hir: Bool): SodigyCompiler = if self.is_dump_hir_set {
  panic("flag `--dump-hir` is used multiple times")
} else {
    self `dump_hir dump_hir `is_dump_hir_set True
};

@method(SodigyCompiler)
let check_validity(self: SodigyCompiler): SodigyCompiler = ...;  # TODO: throw an error when something's wrong

@allow(Warnings.UnnecessaryTypeConversion)
let output = SodigyCompiler.base().input("a.out").dump_hir("true" as Bool).check_validity();
```

부가적인 효과: Sodigy Compiler 안에 Sodigy 코드가 많이 들어가면 들어갈수록 좋음!

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

---

`as` operator

- Rust에서는 infallible하게만 쓸 수 있음
- zig에서는 예외 가능한 거랑 아닌 거랑 구분하지 않나?? 아닌가??

내 생각이 -> `A as B` 했을 때 `B`가 나오는 거 하고 `Result(B, _)`가 나오는 거를 둘 다 쓰고 싶음!

1. infallible한 정의만 있는 경우
  - 전자는 그냥 `B`를 주면 되고, 후자도 그냥 `B`를 구한 다음에 거기에 `Ok()`를 씌우면 됨!
2. fallible한 정의만 있는 경우
  - 전자는 무조건 `.unwrap`을 호출하고, 후자는 그냥 fallible한 거 호출하면 됨
3. 둘 다 있는 경우
  - 전자 후자 각각 호출

fallible한 거는 어떻게 나타냄? 이것도 infix operator로 나타내?

1. `A !as B` -> 보기에 너무 이상함...
2. `A try_as B` -> 새로운 keyword를 추가해야하는데 그 정도의 가치가 있는지 모르겠음...

둘 다 굳이...

아니면 `A as Int`, `A as Option(Int)`로 해도 되지 않음??

---

methods

```
@method(Node(T))
let get<T>(self: Node(T), key: Int): T = { ... };
```

1. `get`의 첫번째 arg의 type은 `Node(T)`이어야 함. 첫번째 arg의 이름은 상관없지만 conventionally `self`라고 씀
  - `@method(Node)`라고 쓰면 안됨! `@method(Node(T))`라고 써야 함! 그래야 `@method(Node(Int))`같은 표현도 가능하거든...
  - `@method(Node)`라고만 쓰면 자동으로 `@method(Node(T))`라고 생각하게 할까??
  - `@method(Node)`라고만 쓰면 `self`의 type을 가져오도록 할까?
2. MIR이 `node.get(3)`을 발견하면 `node`의 type을 검사한 뒤, `get`이라는 field가 없으면 자동으로 `get(node, 3)`으로 desugar함.
  - `get`이라는 field가 있으면? 그거는 `@method(Node(T))`를 처리하는 친구가 에러를 날릴 거임!
  - 단순히 `get(node, 3)`으로 desugar하는게 아니고, 좀 더 복잡한 이름을 쓰는게 나을 수도? `[].get(3)`하고 `node.get(3)`하고 이름 충돌이 있으면 안되잖아. 근데 또 이거는 어차피 uid로 구분할 거니까 상관없을 거 같기도 하고...
3. 사용자가 `get(node, 3)`이라고 쓰는 거는 안됨.
4. `get<T>(self?: Node(T))`도 됨??
  - 이게 돼야 `tree.sdg`가 작동함.
  - 이게 되려면 `@method(Questioned(Node(T)))`가 안되게 만들어야 함!
  - 저걸 금지시켜야 `node?.get()`을 했을 때 안 헷갈리지...

---

HIR collects `Func`... how is it passed to MIR?

1. a function in MIR must be able to have access to all the other functions
2. everything name must be resolvable
  - for example, `c.d` with `import a.b.c;` is converted to `a.b.c.d`, and there must be information of `a`, `a.b`, `a.b.c` and `a.b.c.d`
3. consume HIR sessions from multiple files one by one: lower all the hir funcs to mir funcs
  - once the funcs are lowered, it first infers and checks type
  - the first infer can be imperfect. it runs a few more times until everything makes sense

---

multiple input files...

생각해보면... C는 Input file 사실상 하나만 가능, Python은 input file 무조건 하나만 가능, Rust도 input file 사실상 하나만 가능!

얘도 그냥 input file 하나만 가능하게 해버리자! 그대신 `import`나 `module`이 들어있으면 동일 dir 혹은 하위 dir을 자동으로 뒤지는 거임! 어느 dir을 뒤질지는 컴파일러가 알아서 결정함, user는 컴파일러한테 알려줄 필요없음

예를 들어서, `A.sdg` 안에 `module B;`이 들어있으면, `./A/B.sdg`가 존재해야함! (`./B.sdg`는 안됨?)

`import`하는 거는 3가지 방식, `import A.B;`가 있다고 치면...

- 현재 파일에 `module A;`가 선언되어 있는지를 가장 먼저 확인
- 컴파일러 옵션으로 외부 라이브러리를 줄 수 있게 하자, 어떤 식으로 줄지는 미정
- Go에서는 같은 dir에 `go.mod`가 있어야하지? 그것처럼 동일한 dir에 특정 파일이 있으면 걔가 dependency 알려줄 수도 있음!

위에 3가지를 저 순서대로 해보자!
