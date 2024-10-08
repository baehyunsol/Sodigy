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
  - like `.2f` in Python

1. it's very tough to add room for such markers in f-strings
  - `f"\{ratio:.2f}%"`: is there any possiblity where `:` is used in expressions?
2. why don't you just `f"\{format(ratio, ".2f")}%"`?

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

adding an example code to the document:

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
@method(SodigyCompiler)
let input(self: SodigyCompiler, file: String): SodigyCompiler = self `input self.input.push(file);

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

Type Classes

https://smallcultfollowing.com/babysteps//blog/2016/09/24/intersection-impls/
https://smallcultfollowing.com/babysteps//blog/2016/09/29/distinguishing-reuse-from-override/
https://smallcultfollowing.com/babysteps/blog/2016/10/24/supporting-blanket-impls-in-specialization/
https://smallcultfollowing.com/babysteps/blog/2022/04/17/coherence-and-crate-level-where-clauses/
https://aturon.github.io/tech/2017/02/06/specialization-and-coherence/
https://github.com/purescript/documentation/blob/master/language/Type-Classes.md
https://github.com/Ixrec/rust-orphan-rules

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

Type-infer And Type-check

- https://smallcultfollowing.com/babysteps/blog/2017/03/25/unification-in-chalk-part-1/

Lower to Mir을 하면서 Type::HasToBeInfered를 만나면 걔네한테 전부 id를 붙이셈.

예를 들어서 `let foo = 3;` 하면 `foo`에는 `HasToBeInfered(1234)`가 붙어있고, `3`에는 `Int`가 붙어있겠지.

또, 저기서 `HasToBeInfered(1234) = Int`라는 equation이 나오지? 이 equation들을 전부 table에 저장
-> 나중에 이 table로 모든 `HasToBeInfered`의 type을 알아낼 수 있으면 성공!

더 많은 단계로도 가능

```
let foo: List(Int) = {
  let v = [];

  v
};
```

`v`의 type이 `HasToBeInfered(1234)`로 찍히고, `HasToBeInfered(1234) = List(Placeholder)`가 추가되지? 또, 함수의 끝부분에서 `HasToBeInfered(1234) = List(Int)`가 추가됨 (`v`의 type과 `foo`의 type이 같아야 하니까). 이걸로 `v`를 풀 수 있음!

근데 얘가 `Placeholder`랑 궁합이 별로임... 지금 내 생각은 `[]`에 `List(Placeholder)`를 주고, `None`에 `Option(Placeholder)`를 주는 방식이었잖아? 근데 컴파일 과정에서 모든 값의 type이 정해지도록 해야할 듯...

지금의 계획

1. lower_to_mir 하면서 만나는 모든 type에 일단은 `Placeholder`를 줌. 명백한 애들만 `Int`, `List(String)`처럼 solid type을 줌.
2. expr들을 쭉 순회하면서 `Placeholder`를 `TypeVariable(u64)`로 치환, equation들을 계속 뽑아내면서 inference 시도
  - type inference는 그때그때 조금씩 해야함. 나중에 한꺼번에 하면 equation 개수가 너무 많아질듯.
    - 만약 `TypeVariable(3) = Int`라는 equation을 발견하면,
    - solved_equation에 `TypeVariable(3) = Int`를 추가하고,
    - equation에서 `TypeVariable(3)`을 찾아서 걔네를 전부 `Int`로 substitute하고,
    - 이 과정에서 새로운게 풀릴테니 계속 반복
    - 나중에 solved_equation들을 갖고 실제 Mir::Expr에 있는 type들도 풀어줘야함!
  - type inference를 하다보면 type error도 잡을 수 있을 듯? 예를 들어서 `TypeVariable(3) = Int`하고 `TypeVariable(3) = String`이 동시에 있으면 빼도박도 못하게 오류잖아? 근데 저 equation만 보고 error message를 어떻게 만듦?
  - `let foo = { let v = []; v.push(3) };`가 있다고 치면, `foo`의 type이 `TypeVariable(0)`이고 `v`의 type이 `TypeVariable(1)`이고 `TypeVariable(1) = List(Placeholder)`가 추가되고, ... `v.push`는 어떻게 처리함? 지금 구현으로는 `v`의 type을 모르면 아무것도 못하는데?? 일단 `List(Placeholder)`만 가지고 type class solver가 작동해야할 듯...
    - `List(Placeholder)`가 아니고 `List(TypeVariable(2))`라고 해야하나??

generic은 어떻게 해야할지 전혀 감도 못잡겠음.. 일단 https://rustc-dev-guide.rust-lang.org/generic_arguments.html 도 읽어보고 rustc 코드도 읽어보자!

근데 생각해보면 아직은 type-inference단계잖아? `let foo = [1, 2, "abc", []]`가 있으면 `foo`의 type은 infer해야하지만 (annotation이 없으니까), rhs는 안 건드려도 되는 거 아님?? 즉, type annotation이 있어야 하지만 생략된 자리들만 채우면 되는 거 아님? 그게 type-inference잖아... 그 상태에서 type-check 부르면 type annotation이랑 실제 type이랑 같은지 전부 확인하는 거지 -> 모든 값들이 다 annotate 돼 있으니까 저 Check가 훨씬 쉬운 거 아님? scoped가 됐든 top-level이 됐든 `let foo(x: T1, y: T2): T3 = bar(x, y)`의 모양만 맞추면 됨!

근데, 그럼 모든 Mir::Expr이 type field가 필요해? 소수의 expr만 type info가 필요하고 나머지는 그때그때 type check 하면 되는 거 아님? 굳이 항상 들고 있어야 해??

---

The final stages of the compiler

1. LLVM IR (or cranelift)
  - The runtime of Sodigy is too big to implement in LLVM IR
  - I have to study LLVM anyway
2. C
  - the most reasonable choice for now
  - I need an extra C compiler
    1. depend on a C compiler that is written in Rust
      - https://github.com/onehr/crust
      - https://github.com/jyn514/saltwater
      - https://github.com/ClementTsang/rustcc
      - https://github.com/PhilippRados/wrecc
      - All the above are half-broken
    2. use MakeFile
      - does it work on Windows?
      - does it have to?
    3. embed a very small c compiler in this project
      - https://github.com/rui314/chibicc
      - https://github.com/TinyCC/tinycc
      - https://github.com/drh/lcc
        - its license is too restrictive
3. How about Zig?
4. Binary
5. Transpiling Sodigy to Python/Javascript
  - Very slow (even slower than naive Python/Javascript), but would be very useful

---

Better Document

let's get some inspirations from https://ziglang.org/documentation/0.11.0/

- it has many erroneous code snippets
  - since I have Sodigy syntax highlighter for html and error syntax highlighter (I can directly generate htmls from the dumped json), I can automatically add erroneous sodigy code snippets and their error messages

---

sketch for package manager

```
/src
  /main.sdg
  /util.sdg
  /runner.sdg
  /util
    /web.sdg
    /app.sdg
  /runner
    /web.sdg
    /app.sdg
```

0. each file defines module correctly, and each file imports all the other files (except main)
1. `sodigy -H ./src/main.sdg -o ./target/hirs/main.hir`
  - the package manager also records the hash of `main.sdg`, so that it doesn't run the hir pass again if the source code hasn't changed
2. the compiler tells you that `main` defines modules named `util` and `runner`
  - there must be a cli option for this. compiler has to dump the list of the names after the hir pass
3. `sodigy -H ./src/util.sdg -o ./target/hirs/util.hir` and `sodigy -H ./src/runner.sdg -o ./target/hirs/runner.hir` in parallel
4. the compiler tells you that `util` defines modules named `web` and `app`, and `runner` defines modules named `web` and `app`
  - it also tells you that the files import `module.util.web`, `module.util.app`, ... and the likes
  - the compiler only cares about the first elements of each path: the compiler can never know whether `util` in `module.util.web` is a function or a module (but it's guaranteed that the first element is a module)
5. iterate step 3 and 4 until all the source files have hir files
6. `sodigy -M ./target/hirs/main.hir -o ./target/binary/main -L util=./target/hirs/util.hir runner=./target/hirs/runner.hir`
  - TODO: how would I link `./target/hirs/util/web.hir` and the likes?
    - 저걸 호출하는 시점에서는 module.util.web에서 어디까지가 module이고 어디부터가 function인지 알 수 있음 (다 돌면서 hir 만들면서 기록해뒀으니까)
    - 그러면 `-L util.web=./target/hirs/util/web.hir` 이런 식으로 다 추가해줘도 괜찮을 듯?
    - 파일 하나하나 추가하는게 좀 비효율적이긴 함. Rust를 예로 들면, 각 crate마다 해당 crate를 나타내는 파일을 하나만 쓰거나, 현재 crate의 모든 rlib이 들어있는 dir 하나만 넘기거나 그렇게 함...
      - 추가 flag를 만들어서 `./target/hirs`까지만 넘기면 그 안에서 알아서 이름 보고 찾게 할까?
      - std 생각하면 이게 맞음...
  - 저러면 어떻게 도는데? 일단은 main.hir을 돌면서 foreign name을 찾을 거고,
    - 각 foreign name에 대해서, `-L`로 준게 있는지 확인을 할 거고,
    - `-L`에 hir이 있으면... then what?

import rules for package manager

1. `util/web.sdg` can import `runner/web.sdg` by `import module.runner.web;`
  - is `module` proper keyword here?
  - it has to prevent users from defining a module named `module`
  - does it also have to prevent users from defining a module named `std`?
2. `util/web.sdg` can import `util/app.sdg` by `import module.util.app;`
  - we need something like `super` keyword in Rust
3. `util.sdg` already imports `util/web.sdg` implicitly because it has `module web;` in it

---

아희 interpreter in Sodigy

1. very thin wrapper on 아희
  - each character in the grid is translated to a unique function
  - the main function calls the function of the first character, and that's it
2. no optimization at all
  - not in the interpreter
  - what I expect is, optimizations in Sodigy would remove all the redundancies in the interpreter

---

I don't like the term "scoped let" in the error message "unused local name binding in a scoped let: `x`"

---

naming conventions (in Sodigy and the Compiler)

into_XXX vs to_XXX vs as_XXX

---

prefixed strings

1. `b`, `f`
  - already implemented
  - bf doesn't make sense
2. `r`
  - necessary
  - `br` makes sense but `bf` doesn't
  - it has to modify the lexer and parser because escape rules are hard-coded
  - it also has to modify the parser because the prefix characters are hard-coded
3. something for regex (how about `re`?)
  - my current plan is to allow regex literals only in patterns
  - it assumes `r`. there's no `e` prefix, it must always be used with `r`
  - it also has to modify the parser because the prefix characters are hard-coded

conclusion: modify the lexer and parser so that

1. a string literal can be prefixed with any identifier
  - invalid_string_identifier is raised after lexing
2. more flexible rule for escaping
  - it doesn't care about the escapes while lexing

---

the final path

1. tail call optimization: necessary
2. call stack for non-tail calls: my own vs C's (or whatever backend it's using)
  - later would be better
3. lazy initializations in scoped lets
  - will try my best to opt out, but cannot remove all
  - relying on the callstack is not 100% safe, there's a niche case where it can be a problem
    - there's `foo(x)` and `x` has to be lazily initialized, but not initialized yet
    - the call to `foo(x)` is not tail-call
    - x may or may not be used in foo
    - there are 2 choices
      - initialize x before calling foo: easier to implement, but could be inefficient in some cases
      - initialize x inside foo: harder to implement
      - even more difficult: let the compiler analyse whether foo uses x or not
    - the best way is to inline the function, but we cannot inline all functions
4. reference counter
  - reference counter on all types, including small integers, booleans, ascii characters... wouldn't that be too inefficient?
  - if there's a string with 100000 characters, which is relatively light in Rust, it has 100001 reference counters in Sodigy
  - how about using small integer types?
    - 1, it only exists in the compiler. Sodigy users have nothing to do with the type
    - 2, if a value of an integer (-ish type) is guaranteed to be in i32::MIN ~ i32::MAX, the compiler uses the type
    - 3, it doesn't use heap and doesn't use the reference counter
    - 4, can be used for: enum variant indices, characters, and small integers (normal integers in Sodigy, but verified to be small by the compiler)
5. malloc/free?
  - with the previous section in mind, now the language has 3 primitive types: big integer, small integer and list
  - big integers and lists have to be malloc/free -ed.

---

another idea on enum: merging enum variant indices

let's say enum `Foo` uses index 0, 1, and 2. Let's say enum `Result` uses index 0 and 1. Then how would a `Result(Foo, Foo)` look like?

```
# Ok(Foo::Zero)
@@enum_variant(
    0,
    @@enum_variant(
        0,
        (),
    ),
)

# Err(Foo::One)
@@enum_variant(
    1,
    @@enum_variant(
        1,
        (),
    ),
)
```

It makes sense, but is a bit inefficient (double boxing). Since there're 6 (2 * 3) variants of `Result(Foo, Foo)`, using a single-layered `@@enum_variant` with index 0 ~ 5 would be enough

---

sodigy regex compiler

0. things that I want sodigy-regex to implement
  - `[...]`, `[^...]`
    - I can use internal representation of these to represent `.`
  - `(...)`, `(?:...)`
  - `(?P<name>...)`
    - is necessary if I'm to allow match statements use regex patterns
  - `(?#...)`
    - so easy to implement hahaha
  - `PAT | PAT`
  - `PAT{m,n}`, `PAT{m,n}?`
  - `\number`
  - `^`, `$`, `\A`, `\Z`
    - the latter two always match the start and the end of the string, while the first two may match the start and the end of the line
  - things that I'm not sure
    - `(?P=name)`: back-reference
      - I'm not sure if I'm smart enough to implement it in Sodigy
    - `(?aiLmsux)`: setting flags
      - seems useful and necessary, but I don't 100% understand what each flag means
    - lookaround: https://elvanov.com/2388 , https://www.regular-expressions.info/lookaround.html
      - `X(?=Y)` X if followed by Y
      - `X(?!Y)` X if not followed by Y
      - `(?<=Y)X` X if after Y
      - `(?<!Y)X` X if not after Y
      - if `X` is matched, it tries to match `Y` without consuming `Y`
1. lower regex
  - `+` -> `{1,}`
  - `*` -> `{0,}`
  - `?` -> `{0,1}`
  - `\d` -> `[0-9]`
  - `\D` -> `[^0-9]`
  - `\s` -> `[ \t\n\r\f\v]`
  - `\S` -> `[^ \t\n\r\f\v]`
2. construct AST
3. construct FSM

Note: you can find Python's test cases at `cpython/Lib/test/test_re.py` in their repo -> it would be safer to import test cases from the file, than creating my own. Creating test cases using Python impl as a reference may fail (there can be bugs), but the test cases, especially something like `test_bug_24680` are not likely to have bugs.

---

REPL mode

1. I need this for quick testing
2. If a line starts with `let`, the definition is added to... somewhere
  - there must be some kinda incremental compilation for this
3. If it's not `let`, it's wrapped with `let main = print(` and `);`
  - the main function is abandoned after an execution
4. there must be multi-line mode
  - for example, Python waits until I close a parenthesis if I open one in a repl

---

reasonable syntax for type classes, like `<T: Clone>` or `where T: Clone` in Rust

1. It's too early to define how type classes work
2. Let's at least define the syntax for them. Flexible enough to embrace future improvements
  - It's tough to put something inside the angle brackets: the parser does not group the angle brackets.
  - are type classes an expression? if so, I would need an extremely flexible syntax
  - `where` style would be better, or how about decorators?

---

version에다가 git hash도 넣고 싶음. git commit 하기 전에 검사하는 스크립트를 짜야하나?

---

Is `!` really a pure-type? In Rust, the order of execution (which line is executed after which line) is explicit, and it's clear whether `panic!` is invoked or not. That clearity makes every analysis, including type-checking, possible.

But in Sodigy, it's not sure whether a `let x = panic()` is evaluated or not. That depends on compiler's optimization and runtime implementation. If a behavior of a program depends on the version of the compiler/runtime, can I call it pure?

---

what is impossible in rust but has to be possible in Sodigy:

```
match foo() {
  Foo::String(n)
  | Foo::Int(n) => n.to_string(),
}
```

let's say `n` is `String` or `Int`. It's impossible in Rust because they have different types. But since both implements `.to_string()`, I want that code to be valid.

---

unused import warnings

1. 원래는 hir에서 name collect가 전부 다 돈 다음에 unused import warning을 날렸음
2. 이제는 name collector가 없음
3. mir의 dependency graph는 unused import를 못 잡음 (inter-function 검사를 못하니까)
4. 나중에 unused function 검사하면서 같이 검사해야하나? 이것도 조금 tricky한게, mir에서는 더이상 파일 단위의 구분이 없지만, unused import는 파일 단위로 검사해야함...
5. 생각해보니까 hir name resolving하면서 자연스럽게 될 듯? name resolving 하면서 unknown ident를 전부 찾아서 uid를 붙이는데, unknown ident는 전부 import 문에서 찾을테니까! 중간에 check 한번만 하면 될 듯!

---

Come to think about it, cloning (Rust's clone, not Sodigy's) `hir::Expr`, `Expr` per se, is very dangerous. If there's an anonymous function in an expr, it would create a new `Func` instance. If the expr is cloned 3 times, there's a possibility that the func is also instantiated 3 times...

TBH, I'm not smart enough to design and implement a *perfect* compiler (and language), the only way is to run enough tests and make sure that it covers all the edge cases

---

How would I lower `"abc": String` in hir to mir?

1. uid of `string_init`
  - what does `string_init` do?
2. `['a', 'b', 'c'].into(String)`
  - can we have an optimization? It would be very painful long strings, and the long strings would be more common than I could think
    - let's not think about optimizations for now. Let's just focus on making it work.
  - do I have to wait until TypeClasses are stable? `into` works with type classes
  - type classes? I don't even have specs for methods...

---

Why I need very sophisticated type classes + why people don't like `List.map` in Elm.

Rust implements map, filter, count, sum and many other methods on `Iter`, not on types. That means we don't have to impl `HashMap.iter.filter` and `HashSet.iter.filter` separately. But for elm (and Sodigy for now), people have to impl `List.map` and `Set.map` separately, while the impl of the two would be very similar.

How would I implement map function?

1. I want it to be a method, rather than a normal function: `a.map().filter().sum()` is nicer than `sum(filter(map(a)))`. Imagine nesting it.
2. It has to be generic. As I just mentioned, I don't want to inherit the disadvantages of Elm.

---

lang_item vs prelude

1. if users can access the name, that's prelude. otherwise, that's lang item
2. ... no it's not. `@@_lang_item_type` and `Prelude::Type` are supposed to be the same
  - the difference is that it's dangerous to use an identifier `Type`, so the compiler just generates an identifier with special characters
