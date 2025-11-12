# 76. Subtyping...

1. Never type만 고려할 경우
  - `Never` is a subtype of everything
  - `Never`를 위한 variant와 (`Type::Never`) notation (`!`)을 새로 만들어야 함
  - assertion이나 if처럼 특정 type을 기대하는 경우: 해당 type의 subtype이 나오면 맞다고 하고 넘어가기
    - 함수 arg도 이에 해당
  - list처럼 여러 type이 동일하기를 기대하는 경우
    - 각 type을 전부 subtype으로 묶은 다음에 가장 concrete한 type을 만들어서 전체의 type으로 처리
    - 묶는데 실패하면 오류
  - `TypeVar(x) = Type::Never`인 경우
    - 살짝 꼼수를 씀. 일단은 `TypeVar(x)`를 안 풀고 남겨놔. 나중에 더 자세한 type을 찾으면 `TypeVar(x)`를 풀고, 끝까지 안 풀리면 그냥 `Type::Never`를 넣는 거지.
  - `TypeVar(x) = Result<Int, !>`인 경우
    - 이거는 어쩔 수 없다 ㅠㅠ
    - 아니면, `TypeVar(x) = Result<Int, TypeVar(new)>`로 한 다음에 `TypeVar(new) = !`를 추가로 대입하는 방법도 있음..!!
      - 아니지, 이거를 해도 `TypeVar(new)`를 풀 방법이 없지. 다른 곳에서 등장을 안할텐데?
2. general subtyping

# 75. inter-hir

inter-hir이 너무 더러워지고 있음. 걍 싹다 날리고 새로 짤까?

1. 현재 문제: `a.b.c`가 있으면 `a`의 kind를 확인해서 module일 경우 `b`까지 풀어야함. 지금 이게 전혀 안되는 중. 만약 `b`가 alias면... 문제임. 즉, alias를 풀 때 이것도 같이 해야함.
2. 현재 또다른 문제: `resolve_type`하고 `resolve_name_alias_in_type`을 따로 하니까 코드가 더러워짐...

해야되는 거 총정리

1. `use x.y.z as w;`가 있을 경우 `w`를 다 찾아서 `x.y.z`로 바꾸기
  - name alias 안에서 찾기
  - type alias/annotation 안에서 찾기
  - expr 안에서 찾기
2. `type MyOption = Option<Int>;`가 있을 경우 `MyOption`을 다 찾아서 `Option<Int>`로 바꾸기
  - name alias 안에서 찾기
  - type alias/annotation 안에서 찾기
  - expr 안에서 찾기
  - 각 context마다 param 고려할게 상이함... 잘 고려해야함 ㅠㅠ
  - param이 있을 경우 param을 끼워넣는 작업도 여기서 해야함
3. `x.y.z`에서 `x`가 module이거나 enum일 경우, `y`의 def_span을 찾기
  - name alias 안에서 찾기
  - type alias/annotation 안에서 찾기
  - expr 안에서 찾기
  - `x`가 module일 경우, `y`의 visibility도 이때 검사해야함
    - 단, field나 method의 visibility는 아직 못 보고 module item만 검사 가능
    - 이 expr/alias/annotation이 속한 module에서 `y`를 볼 수 있는지 없는지를 확인해야함

- 저 3개가 서로 엮여있을 수 있으므로 동시에 해야함.
  - alias 안에서만 먼저 찾고 expr, type annoation 안에서 찾으면 시간을 좀 절약할 수 있음
- 저 3개 사이에 infinite loop가 발생할 수 있으므로 검사해야함.
  - 또다른 특이한 edge case: `use x.y.z as a; use a.b.c as x;`가 있으면 (근데 intra-hir에서 못 잡았으면), 저거를 n번 풀면 field의 길이가 2의 n제곱이 됨. 즉, recursion_limit이 20 정도만 돼도 맛이 감. 저거는 미리 탈출해야함!!
- name alias 안에서 무언가를 찾는 함수, type alias/annotation 안에서 무언가를 찾는 함수, expr 안에서 무언가를 찾는 함수를 각각 만들어야함. 그리고 type alias/annotation과 expr은 recursive하게 찾아야 함.

# 74. `#[no_type]`

1. `read_compound`의 경우 아무 값이나 넣을 수 있기 때문에 `Any` type이 필요
2. `panic`의 경우 `Never` type을 구현하거나 아무 값이나 return할 수 있게 하거나...

아니면 손쉬운 trick이 있음: `read_compoun<T, U>(ls: T, i: Int) -> U`로 한 다음에 얘네는 generic이 infer가 안돼도 error를 안 내는 거지!!
panic도 마찬가지: `panic<T>() -> T`라고 한 다음에 generic이 infer가 안돼도 error를 안내면 됨.

이러면 "a type that is a subtype of every type"을 구현할 수 있음!!

생각해보니까 이거 안됨. `fn always_panic() = panic();`을 하면 쟤의 type을 `T`로 추론하겠지? 근데 어디서는 `always_panic`을 int 자리에 쓰고 어디서는 `always_panic`을 string 자리에 쓰면 그 둘이 type collision이 나잖아? 그럼 안되지...

`read_compound`는 저렇게 그대로 써도 될 듯??

# 73. Decorator

Rust랑 비슷하게 만든다치면 decorator도 `#[built_in]`처럼 해야하지 않음??

그럼 이름도 decorator가 아니라 attribute라고 해야하나?? 근데 attribute라는 용어는 이미 쓰고 있는데...

Draft

1. `#[built_in]`, `#[lang_item("blah_blah")]`처럼 하기. 즉, `@` 뒤에 오던 걸 `#[]` 안에 넣는 거임!!
2. Rust에서는 `#[must_use = "You must use this!!!"]`처럼도 쓰는데 이건 못쓰게 막기
3. decorator라는 용어와 attribute라는 용어는 그대로 쓰기
4. decorator이름에 `Vec<InternedString>`대신 `InternedString` 쓰기... please...
  - Rust에서는 path도 사용가능하지만 일단 Sodigy에서는 안되게 막을 거임. 아직은 user-defined decorator가 들어갈 자리가 없거든 ㅋㅋ
5. `#![]`이랑 `//!`도 구현하기?? ㄱㄱㄱ

# 72. Visibility

가라로 하던 거 업보 청산할 시간...

1. 지금은 inter-hir에서 `iter_public_names`를 한 다음에, public한 name들만 module_name_map에 올려둠.
2. 즉, 완전 public한 애들만 resolve를 하기 때문에 딴 애들은 ... 건들지도 않음 ㅠㅠ
3. 일단, private한 애들도 resolve를 하긴 해야함.
4. 지금은 public/private만 구분을 하고 있는데 module 단위로 visibility를 따로 확인해야함!
  - 어느 타이밍에 하지...
  - `a.b`가 있으면 `a`는 확인할 필요없고, `b`는 확인해야함.
  - `a`가
    - module일 경우, `b`는 item이고, attribute 뒤져보면 visibility가 있음. 현재 lowering 하고있는 module (= file)이 저 item을 볼 수 있는지 확인하면 됨.
    - value일 경우, `b`는 field이고, `a`의 type을 알게될 때까지 검사가 불가능. `a`의 type을 알더라도 검사가 좀 빡셈, 얘는 module 단위로 visibility가 있는게 아니거든...
    - enum일 경우, `a`의 visibilty와 `b`의 visibility가 동일하기 때문에 상관없음
      - 아닌가? private variant같은 개념도 만들까?

top-level `let`은 public 선언을 어떻게 함? `pub let`이라고 그래?? 좀 이상한데?? `pub const`라고 그래?? 그럼 `const`랑 `let`이랑 다른게 뭔데?? ㅠㅠ 그럼 싹다 `const`로 통일?? nope... 참고로 rust는 inline block 안에서 `const` 사용가능... -> 이거는 어쩔 수 없는 문제인 듯? purity로 인해서 `let`과 `const`의 차이가 사라진 거고, 이거는 purity가 Rust-like보다 우선순위가 높음!!

# 71. Wildcard

lexer는 Identifer로 잡은 다음에 parser가 Wildcard로 바꾸기 vs lexer가 Wildcard로 잡아버리기!

Wildcard 사용처를 생각해보자

1. pattern matching
  - `_`로 시작하는 이름은 unused_name 안 날리기? -> 이거 구현하면 사실 그냥 identifer랑 다를게 없음
    - 아니다 살짝 다르네, `_`로 name binding 여러개 해도 오류 날리면 안되니까!!
2. function argument
  - `_`로 시작하는 이름은 unused_name 안 날리기? -> 이거 구현하면 사실 그냥 identifer랑 다를게 없음
  - 이것도 살짝 더 생각해야함. `_`로 된 func arg 여러개 선언하면 오류 날릴 거임?
    - 와 rust에서는 `_`로 된 func arg 여러개 선언하는 거 가능하네!!
  - 그럼 `foo(3, _=4, _=5)` 하면 오류 날려야하는데??
3. type annotation
  - 여기서는 좀 special treatment가 필요함! 어차피 special treatment 할 거면 아예 구분하자 이거지

생각해보니까 identifier가 쓰이는 모든 곳을 다 고쳐야함... 흠 좀 빡셀 거 같기는 한데 ㅠㅠ

그럼 `_`로 시작하는 이름은 unused_name 안 날리는 것만 구현하자!

# 69. trait/method/operator overloading

1. std에 `index`가 있고 `index_list`가 있지? 둘을 어떻게 연결시킬까? -> 여기서 모든 고민이 시작됨.
2. operator overloading을 구현할 거임?
  - 이거는 살짝 반대임
3. trait (a.k.a. type class)를 구현할 거임?
  - 언젠간 구현하고 싶긴 함. 다만 지금 하기에는 너무 벅찰 뿐...
4. method는 어떻게 표시?
  - Rust랑 비슷하게 하려면 `impl Person {}`을 하면 됨!
    - 이러면 `self` keyword도 쓰는 거지?? 그러려면 parser를 좀 수정해야함 ㅠㅠ
    - `self`가 안 붙으면 class method가 되나??
    - 그럼 `impl<T> Option<T> {}`하고 `impl Option<Int> {}`하고 구분해야겠네??
      - 그럼 orphan rule도 만들어야 하는 거 아님??
  - 옛날 sodigy에서는 decorator로 type을 연결시키려 했는데... 별로였음!!

# 68. turbofish operator

1. Samples
  - `a.map().collect::<Vec<_>>()` in rust becomes `a.map().collect.<Vec<_>>()` in Sodigy
  - `Vec::<u32>::decode_impl()` in rust becomes `List.<u32>.decode_impl()` in Sodigy
2. Implementation (let's say in AST)
  - separate token vs attribute of ... what?
  - if it's a separate token, it'd be damn difficult to parse `Expr::Call`.
  - how about treating it like a field?
    - 이게 나을 듯..!!
3. Syntactic ambiguity
  - `1.` is a number, `<` and `>` are infix operators and let's say `T` is an integer value. Then `1.<T>` is a syntactically valid.
    - Well, we can avoid this because `1` cannot be an lhs of a turbofish.
  - 사실 이미 Sodigy에 syntactic ambiguity가 존재하거든? 그래서 여기도 syntactic ambiguity를 넣은 다음에 에러메시지를 좀 더 잘 써줘도 됨.

How about just using `::<>` instead of `.<>`?

# 66. Assertion loops

Rust로 코드를 짜다 보면 for문을 돌면서 assert를 할 일이 많음!! 그럼 자연스럽게 assertion note도 for문에서 만들게 됨. 이걸 Sodigy로 하려면??

1. for문 대신 recursion을 해야함. 이건 타협 불가
2. recursion 안에서 assert를 한 다음에 뭔가를 return 해야함. 이게 좀 애매
  - `assert`를 expr로 쓰기? 는 힘듦 -> block 안에서 syntactic ambiguity가 생기거든
3. assertion note나 assertion name을 pragmatic하게 만들고 싶은데, 이건 구현해야함
  - name을 pragmatic하게 만드는 거는 애매. 이름이 겹치면 어떻게 하지?
  - name은 identifier로 받고 note는 expr로 받을까?

# 65. explicit type casts

1. `String(x)`, `Int(x)`처럼 하기!
  - `Byte(x)`를 하면, `Result<Byte, _>`를 반환해? 그건 좀 많이 이상한데?
2. `as` operator
  - 이것도 마찬가지, `300 as Byte`를 하면 `Result<Byte, _>`를 반환해? 그건 좀...
3. `.into()`

# 62. format string

Lexer도 아직 못 짬 -> 너무 복잡해서 아직 손댈 엄두를 못 내는 중

1. rust 방식
  - {} 안에 들어갈 수 있는 token이 아주 제한됨. 그대신 뒤에 arg로 줄 수 있음. arg는 문법이 아주 풍부. 그대신 `format!`과 `println!`이 compiler-built-in임...
2. python 방식
  - {} 안에 들어갈 수 있는 token에 제한이 거의 없음. 그대신 parsing이 빡세고 가끔 비직관적인 일이 일어남 (`:`를 썼는데 의미가 애매해진다든가...)
3. 절충안
  - 몇몇 token만 허용하기? identifier, comma, dot, parenthesis
  - quote랑 curly brace 빼고 다 허용하기? colon도 허용하면 안될듯

# 61. more on purity

How do you define purity?

1. if x = y, then f(x) = f(y)
  - 참고로 user-defined `=` operator랑은 상관없음!! 그냥 overloading 할 수 있게 열어주자.
2. no side effects
  - How do you define side effect?
3. is `panic()` pure?

# 59. Complete new implementation of Bytecode/VM

1. `scalar` (32 bit) vs `compound` (arbitrary number of scalar/compound values) are still valid.
2. There are 4 stacks, 1 register and 1 heap.

```
stack1: func args (scalar)
curr stack frame size: 3
...  v1  v2  v3  _
     ^           ^
     |           |
    sp1          *-- func args for the next call are pushed to here

When it has to read a value, it does something like `stack1[sp1 + i]`
When it calls another function, it pushes the arguments to `stack1[sp1 + 3 + i]`, and adds 3, which is the stack frame size, to sp1, and jumps. After it comes back, it subtracts 3 from sp1.
When it tail-calls another function, it pushes the arguments to `stack1[sp1 + 3 + i]`, and copies the values in `stack1[sp1 + 3 + i]` to `stack1[sp1 + i]`, so that it doesn't have to move sp1, and jumps.
When it returns... it does nothing! There's nothing to drop! Caller is responsible for decreasing the stack pointer, not callee.

stack2: func args (ptr)
curr stack frame size: 4
...  p1  p2  p3  p4  _
     ^               ^
     |               |
    sp2              *-- func args for the next call are pushed to here

It's like stack1, but you have to inc_rc when you push something to this stack.
When it returns, it has to dec_rc of p1, p2, p3 and p4.

stack3: locals (scalar)
It's like stack1.

stack4: locals (ptr)
It's like stack2, but it dec_rc when it leaves a block (or any namescope), instead of returning from a function

register1: return (scalar or ptr)
A function return value is pushed to here. You have to explicitly drop this value, so that the runtime can dec_rc.

heap
```

3. Optimizations
  - The easiest way of removing heap allocations is destructuring structs.
    - For example, by destructuring `{ let p = Person { age: x, name: y }; ... }` to `{ let age = x; let name = y; ... }`, we have removed a heap allocation of `p: Person`.
    - We can do this if `p` itself is not used, but only its fields are.
    - We can do this at MIR.
  - In the current version, when you want to push a constant to `Stack::Call(1)`, you first push it to `Stack::Return` and clone it to `Stack::Call(1)`. It's damn inefficient. You can pass an argument to `lir::lower_expr`, which stack it should push the result. It's not even an optimization. It's just an implementation, but it's a huge gain.

# 58. unnecessary parenthesis warning

심심해서 구버전 Sodigy에서 내던 warning이 뭐가 있는지 찾아봤거든? 그나마 건질만한게 저거밖에 없음.

1. curly brace도 잡기?
  - `if cond {{{var}}}` -> 이런 말같지도 않은 상황을 상상해볼 수도 있음 ㅋㅋ
2. unnecessary한지 아닌지 어떻게 판단?
  - `if (cond) { .. }` -> 이거 unnecessary? 가독성에 도움될 수도 있잖아.
  - `if cond {(var)}` -> 이거 unnecessary? 이건 unnecessary 해보이긴 함 ㅋㅋㅋ
  - `let x = (var);` -> 이거 unnecessary? var가 길면 가독성에 도움될 수도 있잖아...
  - `foo(x, y, (var), z)` -> 이거 unnecessary??

# 57. `mod` and `use`

1. `mod`랑 `use`는 rust와 동일하게 사용
  - 단, inline module은 아직 고민 중
  - `pub use`는 하고 싶지만 아직 미구현
2. `::`는 안 씀. `use`에서도 `.`으로 이름 이어야 함!
3. 파일구조: rust와 동일
  - 단, `mod mod;`는 아예 원천적으로 막을 거임. `mod r#mod;` 해도 안됨. 그냥 저 이름 자체를 막을 거임.

# 56. byte/bit pattern matching

- some drafts
  - `1010xxxx`: 8 bit integer that is in range `160..=175`. The matched integer is in range `0..=15`
  - `1010..xxxx`: an arbitrary size integer that starts with `1010`. It matches the last 4 bit of the integer.
  - No... not this way. It's too confusing.

# 55. `r#keyword` -> implement this in lexer

`fn use(x) = { ... };` 이런 거 보면 "expected an identifer, got a keyword"라고 할 거잖아? 그럼 note로 "use r#"이라고 알려주고 싶음. 지금은 이걸 표시할 자리가 없는데... `ErrorKind::render`를 조금 수정해서 자리를 만드는게 최선일 듯!

# 51. Number type

10분 정도 고민하고 Sodigy-Ratio로 결정을 내림. 고민 과정은 걍 지웠음 ㅋㅋ

- 결정 이유
  1. runtime-impl을 하려면 intrinsic을 엄청나게 많이 추가해야하고, 그럼 backend 추가하는게 엄청 빡세짐
  2. sodigy-impl을 하면 compiler 최적화가 가능 (그래봤자 runtime-impl보다는 느리겠지만...)
  3. float를 쓰면 0.1 + 0.2해서 0.3이 안되잖아? 난 그건 절대 안됨!!
- potential issues
  1. how do we deal with irrational numbers?
    - 지금 생각으로는... `if numer.len() > limit && denom.len() > limit { ratio.cut() }` 해버리자!
  2. We might need low-level operations (e.g. how many scalars does this integer use?)
    - 사실 이거 없어도 구현은 가능... 나중에 생각하자!
  3. range
    - 어차피 cut을 할 거면 그냥 arbitrary size로 해도 되는 거 아님??

# 50. generic functions

```
fn foo<T>(x) = { .. };
fn bar(..) = baz(foo.<Int>(..), ..);
```

- 일단 `foo` 그자체만 갖고 type-check를 하셈. `Type::GenericDef` 있으니까 type equation은 전부 만들 수 있음.
  - 사실 전부 못 만듦. `fn foo<T>(x: T, y, z) = x.do_something(y, z);`이면 `.do_something`에 대한 정보가 아예 없어서 다른 추론도 막힘.
  - 아니면 ``fn foo<T>(x: T, y) = x `field y;``같은 예시도 생각해볼 수 있음!!
  - 아니면 `fn foo<Person>(x: Person) -> Int = x.age;`도 있고...
    - 근데, 50번 이슈랑 별개로 `x.age`라고 돼 있으면 type-checker가 어떻게 풀어야하나? 지금 방식으로는 못 풀 거 같은데?
    - `y = x.age;`인데 `x`와 `y`의 type을 둘 다 모를 경우 (각각 TV_x, TV_y라고 할게), 얻을 수 있는 정보가 2가지임. `TV_x has field "age"`, `TV_y = field(TV_x, "age")`. 둘다 현재 type expression으로는 표현 불가능. 첫번째 expression을 type-infer에 사용 (`age`라는 이름의 field를 갖고 있는 type들만 걸러내기)할 건지도 애매...
    - 이거는 새로운 type-var를 만들어야 함...
  - 아니면 `fn foo<T>(x: T) -> T = bar.<Int, T>(3, x);` 이런 것도 있음 ㅋㅋㅋ
  - 이걸 해결하려면 constraint를 아주 정교하게 만들거나, 중간 type-var를 엄청 만들거나 해야함
    - 중간 type-var를 만든다고 치면은, `T = Int`일 때와 `T = String`일 때의 중간 type-var들을 분리해야하는데, 그것도 빡셈
- `foo`를 type-check하면 `T`에 대한 constraint가 쌓임.
- 나중에 `foo.<Int>`를 발견했지? 그럼 `T = Int`를 한 다음에 `T`에 붙은 constraint를 전부 만족시킬 수 있는지 확인함.
- 만족이 되면 `foo.<Int>` instance를 만드는 거고, 그렇지 않으면 에러를 내야함. 에러 메시지를 만들 때는 `bar` 안에 있는 `foo.<Int>`의 invocation을 콕 찝어줘야함.
- `foo.<Int>` instance를 만들었으면 코드 안에서 등장하는 `foo.<Int>`를 찾아서 걔네를 바꿔줘야함.
  - 이거 할 때 operator도 전부 갈아주자!
  - 그러려면 operator도 일반 generic function처럼 처리해야함. 그러려면 operator의 generic argument의 def_span을 나타낼 방법이 있어야 함!!
  - 이렇게 하면 코드가 훨씬 간단해짐 `infix_op_type_signatures` 이딴 거 없어도 되거든 ㅋㅋㅋ
  - 생각해보니까 이거 하면 `Callable::GenericInfixOp`도 사라짐!!
    - 오
- 근데 어차피 monomorphize를 할 거면, monomorphize 한 다음에 그 안에서 새로 type-check하면 안됨 (C++ 방식)? 이게 덜 복잡할 거 같은데... 이걸 하려다가 포기했던 이유가
  - 1, error message가 난해해짐.
  - 2, generic function body 안에 type variable X가 있다고 하자, 이 function이 instantiate 될 때마다 X가 하나씩 늘어나야함. X들끼리 서로 다르게 type-infer 해야하거든... 그럼 코드가 엄청 복잡해짐.
- Rust 방식은 하고싶지 않음. 그렇게 하려면 trait system을 완전 정교하게 design 해야하거든...

# 48. Compiler & Sodigy std

Compiler가 Sodigy std를 직접 참조해야할 일이 아주 많음

```sodigy
// Sodigy std

// There's no implementation because it's a built-in type.
type Int;

// It's also built-in.
type Char;

// It's not a built-in type.
type String = [Char];

// It has much more flexibly defined than `index_list`.
fn index<T, U, V>(ls: T, i: U) -> V;

// An instance of `index`. It's still a generic function, though.
fn index_list<T>(ls: [T], i: Int) -> T = {
    if 0 <= i && i < ls.len() {
        // TODO: how do we call built-in functions in Sodigy
        __built_in.read_compound(ls, i + 1)
    }

    else if -ls.len() <= i {
        __built_in.read_compound(ls, i + ls.len() + 1)
    }

    else {
        // TODO: error message
        panic()
    }
}

fn div<T, U, V>(a: T, b: U) -> V;

fn div_int(a: Int, b: Int) -> Int = {
    if b == 0 {
        // TODO: error message
        panic()
    }

    else {
        __built_in.div_int(a, b)
    }
};
```

1. type annotation에서 `Int`를 보면 std에 있는 `Int`의 def_span과 연결해줘야함.
  - `use std.Int`를 implicit하게 hir에 넣어주면, `Int`의 def_span이 자동으로 들어옴.
2. type-infer를 할 때 `3`을 보면 `Int`의 def_span을 이용해서 type을 만들어야함.
  - 이거는 자동으로 할 방법이 없음. `Int`에다가 `#[lang_item("Int")]`라고 붙여주고 type-infer engine이 `lang_item("Int")`를 사용하면 됨.
3. type-checking을 할 때는 `Int`의 def_span을 쓸 거임.
  - 이거는 1번과 마찬가지로 자동으로 해결됨.
4. mir expr lowering에서 `a[0]`을 보면 `index(a, 0)`으로 바꿔야함. 또한, `index`의 generic arg로 `T`, `U`, `V`가 있다는 사실도 써야함.
  - 이때, `index`의 def_span과 `T`, `U`, `V`의 def_span이 필요함.
  - 나중에 `index`를 다시 `index_list`로 바꿔야함.
  - `index_list`는 함수 정의가 Sodigy로 돼 있고, 컴파일러가 이 정의를 볼 수 있어야함.
  - `index_list`는 여전히 generic function이므로 generic function 푸는 과정을 한번 더 거쳐야함!
    - `T`, `U`, `V`의 정보가 이미 다 있으니까 풀기 쉬울 듯... 아닌가?
5. mir expr lowering에서 `a / b`를 보면 `div(a, b)`로 바꾸고 generic arg `T`, `U`, `V`를 전부 주면됨!!

참고: Rust compiler를 뒤져봄.

- Rust는 `i32` 같은 애들은 완전 built-in이어서 정의도 없음
- Rust code 뒤져보면 `#[rustc_intrinsic] fn atomic_load();` 이렇게 생긴 애들 있음. body는 없고 signature만 있음. std 안에서만 쓸 수 있대!

몇가지 예상되는 issues

1. 그럼 user가 `Int`라는 struct를 새로 만들면 name-collision이 나는데?
  - 생각해보니까 잘하면 피할 수 있음. namespace가 여러 겹으로 쌓이는 구조잖아? prelude namespace를 가장 바깥쪽에 두면 user-defined `Int`가 prelude보다 먼저 선택됨.
  - 어쨌든 경고는 날릴까?? 이건 모르겠음...
  - 참고로 Rust는 prelude랑 이름 겹치는 정의 있어도 경고 안 날림

# 46. `include_str!`

이거 엄청 요긴함. Sodigy에도 필요...

1. 아무 type이나 받을 수 있게하기?? 예를 들어서, `[Int]`도 파일에서 바로 읽어서 쓸 수 있는 거임!!
  - 흠... 애매함 `[Int]`면 그냥 `.sdg` 파일에 저장하는게 낫지 않음?
2. `include_bytes!`도 할 거임? 아니면 둘 중에 하나만 하고 `.into.<String>()` 해서 compile-time-evaluation 하기?
  - 그럴 거면 `include_bytes!`만 남기는게 맞지. 근데 실제 사용은 `include_str!`이 훨씬 많을텐데??
3. 아니면 이런 거는 ㅇㄸ? (아주아주) 나중에 macro 문법이 완성되고 나면, `@include[path="../README.md", format="string"]`이런 식으로 하는 거임 ㅋㅋㅋ
  - `format="string"`: `String`
  - `format="bytes"`: `Bytes`
  - `format="json", type="[Int]"`: 와 이거는 골때리네 ㅋㅋㅋ
4. 아니면 serde스러운 걸 구현한 다음에 `serde.from_str.<[Message]>(include_str!("../data.json"))` 이런 식으로 하고 저걸 무조건 compile-time에 실행하도록 decorator를 추가하는 거임!!
  - compile-time-evaluation... 아주 흥미로운 기능임. 이거 잘 만들면 매크로를 (거의) 대체할 수 있음
    - ctfe를 하다가 panic이 나면 어케함? compiler error vs 해당 부분 expr을 통째로 `panic()`으로 교체
    - 사용자가 시켜서 ctfe 하는 거면 compiler error 내도 될 거 같고, 사용자 동의 없이 하는 거면 warning만 날리고 `panic()`으로 교체하는 것도 괜찮을 듯?

# 45. Package manager

1. Are you using git?
  - If so, we either have to link git (like git2), or implement it from scratch (like gix)
  - If not, we have to implement A LOT OF things from scratch
    - we need a safe and secure way to distribute text files (and sometimes binary files)
    - we need versions of the files because the user might want to specify a version of the package
2. Are you caching hir and/or mir?
  - If so, I need an extra layer on top of git.
    - That means we need a server...
  - If so, who is responsible for generating hir/mir?
    - It'd be too expensive for the server to generate all the irs
    - If it's generated by client... that's a security issue!
  - Even though it caches hir/mir, it still has to store & distribute the source code.

# 43. Locks

interned_string이든 file이든 작업하기 전에 lock 걸고 하고 있음. 지금은 lock 파일이랑 작업 파일이랑 별개거든? 그냥 작업 파일에 lock 걸고 작업한 다음에 풀면 되는 거 아님??

# 41. String & Char & Int & Bytes

Runtime has 2 types: scalar vs compound

1. `scalar` is a type that can be represented in 32 bits (`Byte`, `Char`).
2. `compound` is a compound value of 0 or more scalar or compound values. It's reference-counted.
3. `List`, `Tuple` and `Struct` are all just compound types. An element of a compound type can be a scalar or a compound.
4. `String` is just `[Char]` and `Bytes` is just `[Byte]`.
5. An arbitrary-width integer is also just a compound type.
  - Each scalar value can only be accessed by the runtime, and it represents a digit.
6. Issue: in order for the runtime to free allocated memory, it has to know whether a value is scalar or compound. But who stores such information?
  - The bytecodes must inform the runtime how to free the memory.
  - There are only 3 cases:
    - It's scalar, so there's no need to free.
    - It's compound, and it has to drop all the elements.
    - It's compound, and some elements are scalar and the others are compound.
    - Oh, it has to be recursive... hence infinite cases!

# 40. Map

1. In Sodigy vs Builtin
  - Sodigy로 짜면 10배는 느릴 듯 ㅋㅋㅋ
  - builtin 사용하면 언어마다 명세가 조금씩 달라서 애먹을 듯
    - 예를 들어서, 모든 함수가 pure해야하지만 (당연히 backend가 달라도 결과가 같아야함), edge case가 엄청 많을 듯?
2. pattern matching for maps?
  - `if let { "name": name } = map {}`
  - 괜찮을 거 같긴한데? ㅋㅋㅋ
  - 그대신 key 자리에 const만 가능함. `{ r"\d+": number }` 이런 거 안됨
    - 된다고 할까? 구현은 가능하잖아? 속도가 느리면 책임은 프로그래머가 지는 거지
  - empty map도 가능: `if let {} = map {}`이랑 `if map.is_empty() {}`랑 동일! `if let`으로 하니까 이상한데 `match`에서는 쓸만할 듯?
  - length가 1이면 이런 것도 되지 않음? `if let { _: single_value } = map {}`
3. Python style map syntax 추가하면 안됨? `{ "name": name }`
  - 이거 하면 curly brace 쓰는 문법이 3개나 돼버림 ㅠㅠ struct-init, block, map...
  - struct-init을 없애버릴까?
    - struct-init을 parenthesis로 쓰는 것도 괜찮을 거 같기는 한데, 그럼 struct pattern이 애매해짐 ㅠㅠ

# 37. debug function

- 단순 print문이나 log문으로 디버깅하기 -> 필수!
- Sodigy로 서버를 만들면 log를 엄청 남겨야하는데? -> 필..수?

1. `fn debug<T>(v: T, pre: String = "", post: String = "") -> T;`
  - `v`의 값을 출력하고, `v`를 그대로 반환
  - 앞뒤에 추가로 문자열 붙일 수 있음!!
  - 문제점
    - 어느 시점에 호출될지를 정할 수 없음
    - 사용되지 않는 값은 출력이 불가능.
2. `echo` statement
  - 당연히 debug-mode에만 작동
  - 인수를 그대로 출력
  - 문제점
    - statement를 추가하는 거 자체가 별로임
    - debug-mode에만 작동된다는 걸 납득 못하는 사람들이 많을 듯
    - 그냥 print문 대용으로 쓰려고 할 듯
3. breakpoint를 걸 수 있게 할까?
  - 그럼 debugger가 필요한데...
4. 함수 로그 찍는 decorator를 만들까?
  - 진입할 때 arg 전부 다 보여주고, 빠져나올 때 결과값도 보여줌 -> 이러면 tail-call을 못하는데??
  - 진입할 때 arg, datetime만 찍어도 괜찮을 거 같은데??

일단은 보류하고 (아직은 debugging이 필요할 정도로 긴 Sodigy 코드를 못 짬), Sodigy 코드를 많이 짜고 나서 그때 생각할까?

# 36. Sodigy-Shell

There's a shell-script on top of sodigy. It is a completely different language, can call arbitrary sodigy functions (does it?), and is impure.

So, basically, sodigy is a library-only language. If you want to *execute* something, you have to use sodigy shell.

Things that I need: read/write/append to files (including stdin/stdout/stderr). random_int, date, ...

옛날에 sreq에서 하려고 했던 것들 여기서 할까?

1. pipe operator
  - `$in`으로 이전에서 넘어온 값 받게 하자... cause I don't like being implicit
  - `$in`이 들어갈 자리가 명확하면 생략가능하게 하자!
2. args and flags
  - a command takes of small number of args (can be zero) and a lot of flags
  - you can use parenthesis to make args less ambiguous
3. command
  - a command may 1) return a value or 2) fail
    - it must return a single value. there's no tuple in sdgsh
    - if it fails, it might pass `$err` to pipe
4. `or` command
  - if the previous command failed, it's executed
  - if there's no `or` command after a failed command, the entire shell dies immediately
  - can it catch panics?
    - sodigy-shell must be implemented using sodigy bytecodes. so if it can catch panics, we have to allow sodigy bytecodes to catch panics... oh no...
5. calling sodigy functions
  - can the sodigy function take arbitrary types of input?
    - I don't think so...
    - okay types: int, number, string, list of okay types (does it allow list of list of list of integers?)
  - can the sodigy function return arbitrary types?
    - I want the `or` command to take care of sodigy's `Result` type.
    - what if it panics? does the shell die?
    - If we're implementing REPL, it has to be able to return arbitrary types
6. Type checks?
  - the current runtime has no type information... so if we pass an integer to a function that expects a string, it'll behave in really weird way but doesn't throw any error
  - commands have very dynamic types (e.g. return type changes depending on flags)... can we check that?
  - I still want type checks because
    - 1, it provides better error messages, both in compile-mode and REPL-mode
    - 2, I don't want to add runtime type information
  - In order to check types, we need type information (of course), and in order to check types at runtime, we need the type information at runtime...
7. Global variables
  - can we type check this?
  - in order to type check this, users have to annotate the types of global variables... 으악!
8. User-defined commands
  - only a simple macro (text substitution)
9. Interpreted vs compiled
  - if it's interpreted, we have REPL!
  - if it's interpreted, how do we distribute the language?
10. if it's not repl, there must be some kinda entry point
  - it has to read stdin, argv, env vars -> easy as cake
11. comments: `#` vs `//`
  - if it's shell... then we have to use `#` for comments
  - but then, we have to rewrite EVERYTHING, even the lexer...
12. inline expressions
  - bash/zsh doesn't allow that. nu allows that
  - if I have to allow this, the language would be at least 3 times more complicated
13. formatted strings
  - lexing an f-string is a big deal. it's such a big deal that even sodigy can't do that.
  - I don't want to implement such thing again...
14. string escapes
  - it's a big deal to implement escapes in the lexer
  - do I have to do that again...??
15. bash style names (ls, cat, cd, mkdir) vs modern names (read, write, append, list_dir, create_dir, make_dir)
16. dynamically import sodigy functions
  - in order to dynamically import sodigy functions, the sodigy functions cannot use static labels
  - but if the shell is compiled (for distribution), I want them to use static labels
  - how can it dynamically load bytecodes... we need a whole new architecture
  - the user points to sodigy source files, not compiled libraries
    - does it compile the sodigy source on the fly? what if the compilation fails?
    - how does it find the compiled libraries, or check if it exists?
  - `use lib_sth.fn_sth as foo;` -> I want this syntax but I cannot reuse the parser in the sodigy compiler ... :(
17. Then, what happens to the sodigy compiler?
  - `sodigy new <project_name>` creates a new project
  - `sodigy run` in the project dir runs the sodigy-shell file in the project
    - Can a project have multiple shells?
  - `sodigy test` in the project dir runs the tests in the sodigy files
  - `sodigy build` emits an output (C/Rust/Python/Bytecode) (including sodigy-shell)
  - `sodigy interpret <bytecodes_path>` interprets bytecodes
    - The sodigy binary has to be able to run bytecodes anyway (in order to run tests).
18. How about this? Sodigy creating `[Command]` at runtime and the shell executes it.
  - Can we type-check this?
19. Executing arbitrary binary files
  - zsh and nu can execute `~/Documents/Rust/hgp/target/release/hgp`... can it? nope!
20. Another idea for implementation
  - sodigy-shell is just a thin syntactic sugar over sodigy. for example, `ls -l "../" | do_something | do_another_thing $in 3;` is desugared to `do_another_thing(do_something(ls_long(path="../").unwrap()), 3)`.
  - Then, it's passed to the sodigy compiler. Sodigy compiler can do everything. It can even type-check the script.
    - We have to do something with the spans, so that it's error message is readable.
  - The functions in the generated sodigy code are impure, but the compiler doesn't care about that.
  - how does it lower `or` command?
    - `ls -l "../" | or (do_something $err) | do_another_thing $in 3`
    - `do_another_thing(match ls_long(path="../") { Ok(i) => i, Err(e) => do_something(e) }, 3)`
    - Wow... this is strong...

TODO: call it `sodigy-script`, not `sodigy-shell` -> fix everything accordingly

---

다시 생각

1. 지금 컨셉이 너무 애매함
  - shell이라고 하기에는 너무 다르고 (path에 항상 quote해야하고, 기본 명령어 (ls, chdir, mv, cp)들도 다 다르고, arbitrary process 실행도 못하고, 특정 명령어에서 panic이 발생하면 shell 전체가 죽어버리고)
  - script라고 하기에는 너무 약하고 (기본적인 expression도 못씀, 함수 호출 방식도 다름, Python REPL을 쓰던 사람이 sodigy REPL에서 기대하는 거를 아무것도 못함)
  - 아예 다른 언어를 하나 만들고 컨셉을 확실히 해야함 (그게 꼭 shell이나 script일 필요는 없지만)
2. Sodigy에는 없지만 action-language에는 필요한 것들
  - Sodigy는 값 하나만 eval하고 바로 return 하지? action-language는 action 여러개를 연속적으로 실행할 수 있어야함
  - action의 실행 순서를 정할 수 있어야함.
  - 위의 2가지가 생기면 for문도 필요해짐
3. Action-language의 방향
  - 최대한 간단해야함. 복잡한 logic은 Sodigy로 다 구현해야하거든
  - action이 sodigy func를 호출하는 건 가능하지만 반대는 불가능
  - Sodigy와 동일한 VM 위에서 돌아가야함

결론: action 여러개를 연속으로 실행하되, 이전 action의 결과가 이후 action의 실행에 영향을 줌
-> elixir랑 gleam에서 어떻게 구현했는지 좀 볼까...

`read sodigy.toml |> parse |> gen-code |> write -o bin.exe;`

---

How about this? A script language that's very similar to Sodigy, except that,

1. You cannot define new a struct/enum/func/alias, only `let` or `assert`.
2. You can evaluate or execute a function (an action, actually), without `let` or `assert`.
3. `let` and `assert` are always executed in the order
4. You can mutate values
  - `let x = foo();` declares `x` and `x = bar();` assigns `bar()` to `x` (mutates `x`).

# 34. Errors, Panics and Crashes

1. Errors: `Result<T, E>`
2. Panics: `panic(msg: String) -> !`
  - It's impossible to catch a panic.
  - It prints the message to stderr and terminates the process with a non-zero code.
  - Printing a span...??
    - Or... a stacktrace?
    - Stacktrace를 만드려면 runtime을 수정해야하고, 그럼 모든 runtime을 똑같이 구현해야함! 귀찮쓰...
3. Crashes: OOM, Stack overflow, ...
  - 사실 stack overflow도 panic으로 구현하는게 가능함. stack에 뭐 넣을 때마다 크기 검사하는 거임. 그러면 프로그램이 무지 느려지겠지?? ㅋㅋㅋ

모든 예외는 1이나 2를 통해야함. Runtime이 자체적으로 예외를 발생시키는 건 안됨. 예를 들어서 integer division을 한다? divisor가 0인지 아닌지를 Sodigy가 판단을 하고 Sodigy가 panic을 해야함. Python이 ZeroDivisionError를 내는 건 안됨!

Intrinsic 안에서는 safety check를 하지말고, 전부 Sodigy로 구현하자! 예를 들면,

```
// Division
fn div(a: Int, b: Int) -> Int = match b {
    0 => panic("Zero Division Error"),
    _ => Intrinsic.IntegerDiv(a, b),
};

// Array Index
fn index<T>(ls: [T], i: Int) -> T = match i {
    // It has to add 1 to `i`, because the first place of the compound value is for the length of the list.
    i if 0 <= i && i < ls.len() => Builtin.ReadCompound(ls, i + 1),
    i if -ls.len() <= i => Builtin.ReadCompound(ls, ls.len() + i + 1),
    _ => panic("Index Error"),
};
```

이래야 최적화가 더 잘되지 않을까??

아니면, runtime이 자체적으로 예외를 내는 거를 허용하되, 예외 내는 방식을 정해둘까? 예를 들어서, Sodigy에서 stacktrace 만드는 옵션을 켜면 runtime이 예외를 낼 때도 sodigy stacktrace를 보여줘야하는 거임!

# 32. Removing reference counts

https://www.microsoft.com/en-us/research/wp-content/uploads/2020/11/perceus-tr-v1.pdf

언제 하지..?? 이걸 하려면 `inc_rc`, `dec_rc` 명령어를 lir에 노출시켜야 하나? 그러면 다른 optimization (#31)이 힘들어짐.

사용처:

1. rc가 1일 경우, in-place mutation을 할 수 있음!
  - intrinsic만 적용시키면 됨
2. value의 lifetime 내내 rc가 1일 경우, rc랑 관련된 모든 코드를 날려버리고 바로 drop하면 됨

lir까지 다 완성된 다음에 이 분석을 해도됨: alloc을 하는 명령어들 (struct init, list init, ...), rc를 증가시키는 명령어들 (push), rc를 감소시키는 명령어들 (pop)을 전부 추적 가능하기 때문에... 적당히 symbolic execution 하면 될 듯?? 말이 쉽지 ㅠㅠ

# 31. LIR Optimization idea

현재 실행 중인 함수를 f라고 하자. f의 첫번째 arg를 (if exists) x라고 하자. `xBC`는 `0 or more bytecodes`를 나타냄!

1. f의 arg의 개수가 N개인데, f 안에서 함수 호출할 때 arg를 N개 미만으로 사용할 경우 (예시에서는 N=1로 가정)
  - 원래는 `copy r0 to local0 -> pop r0 -> xBC -> pop local0`인데 `xBC -> pop r0`로 최적화 가능!
    - 단, `xBC`에서 `r0`를 push/pop하면 안됨.
    - 단, `xBC`에서 `local0`를 사용하는 부분을 찾아서 전부 `r0`를 사용하도록 바꿔야 함.
  - 여기서 핵심은 `copy r0 to something`을 없애는 거임... 얼마나 wild하게 최적화가 가능하려나?
  - 아니면 symbolic execution을 해버려도 됨!
2. recursive call의 경우, local_i -> r_i -> local_i로 옮기는 것보다 local_i에 그대로 두는게 더 나은 경우도 있음!
  - f가 recursive하다고 치면, f_recursion라는 함수를 새로 만드는 거임!
  - 다른 함수가 f를 부를 때는 f를 그대로 쓰고, f가 f를 부를 때는 f_recursion을 사용
  - f_recursion은 arg가 local_i에 담겨있다고 생각할 거임
  - 생각보다 효과가 별로 없으려나..??
3. f가 g를 호출하는데 g의 첫번째 arg가 x인 경우
  - 원래는 `copy r0 to local0 -> pop r0 -> xBC -> copy local0 to r0 -> call g -> pop r0`인데 `copy r0 to local0 -> xBC -> call g -> pop r0`로 최적화 가능!
    - 단, `xBC`에서 `r0`를 push/pop하면 안됨.
    - `xBC`에서 `local0`를 한번도 안 쓰면 `xBC -> call g -> pop r0`도 가능
4. f 안에서 const나 identifier를 읽는 경우 (`x2`를 읽어서 `local3`에 쓴다고 치자)
  - const: `push const to return -> push return to local3`인데 `push const to local3` 해버리고 싶음...
  - identifier: `push x2 to return -> push return to local3`인데 `push x2 to local3` 해버리고 싶음...
  - 위에서 local_i/r_i의 push/pop을 분석하는 것과는 조금 다름. `return`은 stack이 아니기 때문!
5. 간단한 Intrinsic을 실행하는 경우 (let's say IntegerAdd and move the result to `local3`)
  - `eval r0 -> eval r1 -> push call stack -> call IntegerAdd -> pop call stack -> copy ret to local3`
  - runtime에서 이걸 inline 해버리면: `eval r0 -> eval r1 -> add r0 and r1 and push the result to local3 -> pop r0 -> pop r1`
  - 이걸 lir로 표현할 수는 없나... 그냥 intrinsic은 무조건 inline으로 처리할까? 즉, intrinsic 건드릴 때는 call stack 안 건드리고, r_i를 바로 사용한 다음에 pop 해버리는 거지...
    - 이게 맞을 듯? 아무리 비싼 intrinsic이더라도 결국에는 runtime의 callstack을 사용하지, sodigy의 callstack은 건드릴 필요없음!
6. 최적화할 때 추가로 필요한 정보
  - `push r0 -> xBC -> pop r0`를 한다고 치자, 그럼 `xBC`에서 `r0`를 필요로하는지 궁금하겠지? 근데 `call g`라고만 돼 있으면 `r0`를 읽는지 안 읽는지 알 방법이 없음. 결국 `g`가 어느 register를 읽는지를 알아야함. 이거를 1) 거대한 map을 만들어놓고 그때그때 확인한다 vs 2) Bytecode 안에다가 어딘가에 적어둔다.

# 30. C Runtime (or Rust/Python/Javascript)

1. There are only 2 primitive types in the runtime: Scalar and Compound
  - Integer (arbitrary width): compound
    - `[ref_count, n1: scalar, n2: scalar, ...]`
  - String: just `[Char]`
  - Char: scalar
  - Byte: scalar
  - Compound: List/Tuple/Struct
    - Tuple/Struct: `[ref_count, val1: compound, val2: compound, ...]`
    - List: `[ref_count, length: _, elem1: compound, elem2: compound, ...]`
      - TODO: `length`에 sodigy integer를 쓰면 성능이 떨어지고 scalar를 쓰면 구현이 복잡해짐
  - `compound` is a pointer that points to the ref_count of the object
2. Issues in C
  - `scalar` and a pointer to `compound` must have the same size.
  - pointer: Real pointer vs Index (an integer)
    - Pointers have different sizes.
    - If using index, I have to implement `malloc` and `free`.
  - 32bit vs 64bit
    - Pointers are usually 64 bits.
    - If it's 64 bits, a string would waste too many space.
    - It's easier to implement arb-integer with 32 bits than 64 bits.
3. Issues in Rust
  - There's no `goto` in Rust.
    - We have to use a gigantic `match` statement... but I hope the rust compiler can optimize this.
  - It's tricky to manage memory manually in Rust.

# 29. Some optimization

```sodigy
fn fac(n) = if n < 2 { 1 } else { n * fac(n - 1) };

fn fibo(n) = if n < 2 { 1 } else { fibo(n - 1) + fibo(n - 2) };

fn reverse(ls) = match ls {
    [] | [_] => ls,
    [x] ++ rem => reverse(rem) ++ [x],
};
```

It's a very very common pattern. Tail-call optimization won't help it because it has to add/prod/concat all the operands in the stack in the end.

1. Condition
  - The function is recursive.
  - The function has multiple branches.
    - A recursive function without branches doesn't terminate. (TODO: Do I have to raise an error if I can detect this?)
  - The function returns type `T`.
  - One of its branch is `Op(a: T, b: T)` and `a` and/or `b` is a recursive call to the function.
  - The operation is associative.
  - There's exactly 1 kind of operations in the branches.
2. Optimization
  - When it's called non-recursively, it initializes `x = Op::<T>::identity()`.
  - If it reaches a branch which looks like `Op(a: T, b: T)`,
    - if `a` is a recursion, it passes `&mut x` and tail-calls the recursive function.
    - if `a` is not a recursion, it doesn't tail-call `a` and applies the operation with `x` and `a`.
  - If it reaches a branch that has type `T`, it just evaluates the value (not tail-call) and applies the operation to `x`.

# 28. Test & Assert

1. Top-level assertions
  - It's like `#[test]` in Rust.
  - In test mode, it checks all the assertions.
    - How do we implement the test runner? If we implement it in Sodigy, it cannot handle panics.
  - Lowering assertions: `assert x == y;` into `if x == y { True } else { panic() };`
    - Who is responsible for this lowering? Anyone, even AST can do this.
      - But I prefer doing it after type-checking
      - lir will do this -> 지금은 eval 해서 boolean 값을 `Register::Return`에 넣고 `Bytecode::Return`을 호출하는데, 이걸 바꾸자. eval 해서 panic 하거나, 아무일도 없거나 (레지스터도 다 원상복구)
        - 이렇게 하면, inline assertion이든 top-level assertion이든 그냥 bytecode 그대로 읽으면 됨!!
    - Panic message: name (if exists), span (row/col), span (rendered), values (if possible)
    - I prefer panicking when the assertion is failed, than returning False because
      - there's no way to check the value of inline assertions
      - an erroneous test might panic, so we have to somehow catch it anyway
2. Inline assertions
  - It's like `assert!` in Rust.
  - In release mode, inline assertions are disabled.
3. Name-analysis: We have to tweak some logic.
  - If a name is only used by assertions, but not by expressions, we raise an unused name warning.
    - But we add an extra help message here, saying that the name is only used in debug mode.
    - How about adding `#[unused]` decorator?
      - Just being curious here,,, is it okay to use a name that's decorated with `#[unused]`?
      - How about `#[allow(unused)]`?
        - well... currently the parser uses expr_parser to parse the arguments of a decorator. But the hir's expr_parser won't allow the identifier `unused`. There are a few ways to fix this:
        - First, we can implement a separate parser for decorators. But then we have to write parser for each decorator. That'd be huge!
          - Hir has to do this. If we choose a right timing, it can access to defined names (if it has to), and use undefined names (if it wants to).
        - Second, we can add `unused` to namespace (only when parsing decorators).
        - Third, we can use `#[allow("unused")]` syntax.
  - If a name is used by expressions only once, and multiple time by assertions, we inline the name anyway. For example, `{ let x = foo() + 1; assert x > 0; assert x > 1; [x] }` becomes `{ let x = foo() + 1; assert x > 0; assert x > 1; [foo() + 1] }`.
    - We need a lot of tweaks here...
    - `let x` statement is removed in release mode, but not in debug mode.
4. Assertions that are enabled in release mode.
  - How about `#[always]` decorator?
  - The compiler treats such assertions like an expression, not an assertion.
  - If a top-level assertion is decorated with `#[always]`, it's asserted before entering the main function.
    - It's ignored in test-context.
5. Syntactic sugar for `assert x == y;`
  - 이게 실패하면 lhs와 rhs를 확인해야함...
  - 근데 syntax 기준으로 뜯어내는 거는 너무 더러운데... ㅜㅜ 이건 마치 `==`를 syntactic sugar로 쓰겠다는 발상이잖아 ㅋㅋㅋ
  - 아니면 좀 덜 sugarly하게 할까? 그냥 모든 expr에 대해서 다 inspect 하는 거임 ㅋㅋㅋ
    - value가 `Call { func: Callable, args: Vec<Expr> }`인 경우, `func`랑 `args`를 dump (infix_op도 다 여기에 잡힘)
    - value가 `Block { lets: Vec<Let>, value: Expr }`인 경우, `lets`를 dump (expr만), `value`는 dump할 필요없음 (당연히 False일테니)
    - value가 `if { cond: Expr, .. }`인 경우, `cond`를 dump, `value`는 dump할 필요없음 (당연히 False일테니)
    - value가 `match { value: Expr, .. }`인 경우, `value`를 dump하고 어느 branch에 걸렸는지도 dump
6. Naming assertions: `#[name("fibo_assert")]`.
7. Test 결과를 runtime이 compiler한테 다시 전달하면 compiler가 span 꾸며서 dump하기... 괜찮은 듯!
  - 지금은 test 돌리면 runtime에서 알아서 모든 test 돌리고 결과물 즉시 출력하게 돼 있거든? 이러지말고,
  - 1, runtime에다가 label id를 주면 runtime이 그 label을 실행하도록 code gen
  - 2, compiler가 runtime한테 label을 하나씩 줌.
  - 3, runtime의 exit code를 보고 실패/성공을 판단
  - 4, compiler가 결과물을 출력
    - 이러면 결과물을 출력하는 코드를 하나만 짜도 됨.
    - 이러면 span까지 같이 보여줄 수 있음!!
      - 사실 top-level assertion은 span이 필요가 없고, inline assertion의 span이 더 중요함. 근데 inline assertion은 span을 바로 출력하는게 좀 빡셈... inline assertion이 error message를 잘 만들어서 던지면 compiler가 그걸 읽고 regex로 뜯어서 span을 찾아낸 다음에 render 해야함...
  - 문제: rust로 구현된 interpreter는 이게 되는데, Python 구현체는 즉시 호출이 불가능 (하거나 힘듦).
    - Python이나 javascript는 어찌저찌 한다고 쳐봐 (python path를 넘겨주는 거지), C는 어케할 건데?
    - 생각하면 할수록, runtime이 알아서 테스트 돌리고 끝나야함...ㅜㅜ
    - `cargo test` 해보니까 얘도 큰 binary 만들어서 그거 한번 돌리고 끝남. 출력도 다 이 안에서 하고, panic도 지가 알아서 잡는 듯?
    - 애초에 backend가 여러개인게 문제임!! 그냥 rust나 Python으로만 구현하고 다른 backend는 나중에 생각해야함...
    - if we can catch panics, we can implement the test harness completely in bytecode...

# 25. Make it more Rust-like!! ... 하다가 생긴 문제점

Name binding에 `$`를 안 붙이니까 한가지 문제가 생김: `True`랑 `False`에 match 하려면 `$True`, `$False`를 해야함... Rust는 `true`/`false`가 keyword여서 이런 문제가 없음.

-> 생각해보니까 이것도 안되네. `$True`면은 "True라는 이름을 가진 변수와 값이 같다"라는 뜻이잖아...

# 24. tuple struct

```rust
struct Point = (Number, Number);
```

# 23. dotdot in struct init

```rust
Person {
  name: "Bae",
  ..Person.default()
}
```

하는 김에 `Person { name: name }`을 `Person { name }`으로 쓰는 syntax sugar도 만들고 싶음.

얘네 하려면 한가지 문제가, 지금은 `{ IDENT COLON .? }`를 확인해서 struct_init인지 block인지 구분하거든? 이게 더이상 안 먹히게 됨. 이게 안 먹히면 `if IDENT { .? }`를 보고 뒤의 group이 true_value인지 struct_init인지 판단할 수가 없음... Rust도 동일한 문제가 있거든? 그래서 얘네는 무조건 true_value로 취급해버림. 만약에 저 위치에 struct_init을 쓰고 싶으면 무조건 괄호로 묶어야함 ㅋㅋ 걍 따라하자 ㅋㅋ

I found that rustc also has an issue. I opend it hahaha: [issue](https://github.com/rust-lang/rust/issues/147877).

# 18. negative index

`a[-1]`을 하면 맨 마지막 element를 주기

1. a에 element가 20개인데 `a[-200]`를 하면 10바퀴 돌아? 아니면 `[-20]` 밑으로는 다 error?
  - Python throws an error for `a[-200]`.
2. `a[2..10]`은 slice로 할 거잖아, 그럼 `a[2..-1]`도 돼?
  - 근데 `2..-1`은 그자체로 runtime error 아냐? 아닌가...
  - Rust에서 `.get(10..2)`로 하니까 `None` 나옴. 즉, `10..2` 자체는 문제가 없음!

# 12. Type inference

```sodigy
fn map(ns: [T], f) = {
    let nx = f(ns[0]);

    if ns.is_empty() { [] } else { [nx] ++ map(ns[1..], f) }
};

// 1. type annotation이 있어야하는 자리에 type annotation이 없으면 추가하고 시작
// - `f: TypeVar(0)`
// - `map_ret: TypeVar(1)`
// - `nx: TypeVar(2)`
//
// 2. f가 callable이라는 걸 확인했으니 arg와 return type에 들어갈 type variable 추가
//    arg가 1개라는 것도 이 시점에선 셀 수 있음!
// - `TypeVar(0) = Fn(TypeVar(3)) -> TypeVar(4)`
//
// 3. `let nx`의 좌변과 우변을 비교해서 추론
// - `TypeVar(2) = TypeVar(4)`
// - 지금 생각해보니 3번 step을 하는 과정에서 2번 step이 나오는게 자연스러움. 그 과정에서 `TypeVar(3) = ReturnType(Op(Index), (List(T), Int))`도 나와야 함!
//
// 4. if문에서 뽑을 수 있는 equation을 다 뽑을 거임. 먼저 cond부터!
// - `ReturnType(Method("is_empty"), (List(T),)) = Bool`
// - 이거는 type var가 없으니 inference 시점에는 의미가 없음. 나중에 type check할 때는 필요하겠지만...
//
// 5. if문의 true value의 type이 함수의 return type과 동일해야함
// - `List(Any) = TypeVar(1)`
// - empty list를 어떻게 표현해야할까? ㅠㅠ
//
// 6. if문의 false value의 type이 함수의 return type과 동일해야함
// - `ReturnType(Op(Concat), (TypeVar(5), TypeVar(6))) = TypeVar(1)`
//
// 7. 6번 과정을 계산하다가 엄청 긴 type expression이 등장했지? 그래서 자동으로 새로운 type var를 정의하고 type equation을 추가함
// - `TypeVar(5) = List(TypeVar(2))`
// - `TypeVar(6) = ReturnType(Fn(map), (TypeVar(7), TypeVar(0)))`
//
// 8. 7번 과정에서 type var를 또 추가함
// - `TypeVar(7) = ReturnType(Op(Index), (List(T), ReturnType(Op(Range), (Int,))))`
//
// 9. 8번의 우변은 즉시 계산 가능
// - `TypeVar(7) = List(T)`

let foo = \() => Some(100);
let x = if let Some(n) = foo() { bar(n) } else { baz };
let y = x + 1;

// 1. type annotation이 있어야하는 자리에 type annotation이 없으면 추가하고 시작
// - `foo: TypeVar(0)`
// - `x: TypeVar(1)`
// - `y: TypeVar(2)`
// - `foo_ret: TypeVar(3)`
// - `n: TypeVar(4)`
//
// 2. `let foo`의 좌변과 우변을 비교해서 추론
// - `TypeVar(0) = Fn() -> TypeVar(5)`
//
// 3. `let x`의 우변에 있는 if문 뜯기, 먼저 cond부터
// - `TypeVar(5) = Option(TypeVar(4))`
//
// 4. if문의 true value와 x가 동일한 type
// - `ReturnType(Fn(bar), (TypeVar(4),)) = TypeVar(1)`
//
// 5. if문의 false value와 x가 동일한 type
// - `baz = TypeVar(1)`
// - 적는 건 이렇게 적었지만 `baz` 자리에 concrete type이 들어가거나 type var가 들어가야함
//
// 6. `let y`의 좌변과 우변을 비교해서 추론
// - `TypeVar(2) = ReturnType(Op(Add), (TypeVar(1), Int))`

fn first(ns) = if ns.is_empty() { 0 } else { ns[0] };

// 1. type annotation이 있어야하는 자리에 type annotation이 없으면 추가하고 시작
// - `ns: TypeVar(0)`
// - `first_ret: TypeVar(1)`
//
// 2. if문의 cond
// - `ReturnType(Method("is_empty"), (TypeVar(0),)) = Bool`
//
// 3. if문의 true_value
// - `Int = TypeVar(1)`
//
// 4. if문의 false value
// - `ReturnType(Op(Index), (TypeVar(0), Int)) = TypeVar(1)`

fn fibo(n) = if n < 2 { 1 } else { fibo(n - 1) + fibo(n - 2) };

// 1. type annotation이 있어야하는 자리에 type annotation이 없으면 추가하고 시작
// - `n: TypeVar(0)`
// - `fibo_ret: TypeVar(1)`
//
// 2. if문의 cond
// - `ReturnType(Op(Lt), (TypeVar(0), Int)) = Bool`
//
// 3. if문의 true value
// - `Int = TypeVar(1)`
//
// 4. if문의 false value
// - `ReturnType(Op(Add), (TypeVar(2), TypeVar(3))) = Int`
// - 이거 하는 시점에 이미 `TypeVar(1) = Int`라고 돼 있으니 그거 반영해서 적었음... 이게 맞겠지??
//
// 5. newly introduced type variables
// - `TypeVar(2) = ReturnType(Fn(fibo), (TypeVar(4),))`
// - `TypeVar(3) = ReturnType(Fn(fibo), (TypeVar(5),))`
//
// 6. newly introduced type variables
// - `TypeVar(4) = ReturnType(Op(Sub), (TypeVar(0), Int))`
// - `TypeVar(5) = ReturnType(Op(Sub), (TypeVar(0), Int))`

fn first(l) = l[0];

// 1. type annotation이 있어야하는 자리에 type annotation이 없으면 추가하고 시작
// - `l: TypeVar(0)`
// - `first_ret: TypeVar(1)`
//
// 2. 함수 body
// - `ReturnType(Op(Index), (TypeVar(0), Int)) = TypeVar(1)`

fn foo<T, U>(a: T, b: U) -> T = a;
let x = foo(100, 200);
let y = foo::<Int, [_]>(100, []);
let z = foo(x, y);

// 1. type annotation이 있어야하는 자리에 type annotation이 없으면 추가하고 시작
// x: TypeVar(0)
// y: TypeVar(1)
// z: TypeVar(2)
//
// 2. `let x`의 좌변과 우변을 비교해서 추론
// TypeVar(0) = ReturnType(Fn(foo), (Int, Int))
//
// 3. `let y`의 좌변과 우변을 비교해서 추론
// foo의 arg로 type equation 한번 만들고 turbo fish로도 type equation 한번 만들자
// TypeVar(1) = ReturnType(Fn(foo), (Int, EmptyList))  // TODO: EmptyList 어케 함?
// TypeVar(1) = ReturnType(Fn(foo), (Int, [_]))
//
// 4. `let z`의 좌변과 우변을 비교해서 추론
// TypeVar(2) = ReturnType(Fn(foo), (TypeVar(0),  TypeVar(1)))

fn first<T, U>(ls: [T], b: U) -> T = ls[0];
let x = first([100, 200, 300], 100);

// 1. type annotation이 있어야하는 자리에 type annotation이 없으면 추가하고 시작
// x: TypeVar(0)
//
// 2. `let x`의 좌변과 우변을 비교해서 추론
// TypeVar(0) = ReturnType(first, ([Int], Int))
// 이것 가지고 TypeVar(0) = Int 만들 수 있음??
```

issues

1. type equation에서 모순이 발견된 경우
  - 즉시 에러 뱉고 죽기: 에러를 하나밖에 못 뱉는다는 문제가 있음...
  - 계속 탐색해서 더 많은 에러 찾기: 에러가 아닌 곳을 에러라고 판단할 가능성이 있음
    - 일단, 현재 equation과 관련된 equation들은 다 죽여놔야함. 안 그러면 비슷한 에러가 엄청 많이 나올 거거든...
    - 만약 컴파일러가 x를 type-infer 하면서 `x: Int`로도 추론하고 `x: String`으로도 추론했다고 치자. `x: Int`가 틀린 추론인데, 이미 다른 부분도 `x: Int`로 unify된 상태임.
      - 이때 `x: String`이 들어오면 이 위치에서 에러가 나겠지?
      - 근데 과거에 `x: Int`라고 잘못 추론한 거 때문에 다른 곳에서도 에러가 줄줄이 나겠지?
  - 에러는 어떻게 만들어? 모순의 종류에 따라 ErrorKind를 따로 만들자!
    - `TypeVar(1) = Int`가 있는데 `TypeVar(1) = String`으로 unify하는 경우
      - `TypeVar(1)`의 def_span으로 가서 얘가 Int인지 String인지 헷갈린다고 하자
    - `ReturnType(Op(Sub), (TypeVar(0), Int)) = String`이 있는데 `Sub`의 정의를 아무리 찾아봐도 저 모양이 안 나올 경우
      - 이게 좀 애매함. "`-` operator로는 ((_, Int), String)이 안 나온다"고 말하면 의미가 없음. 사용자도 이미 알거든.
      - 그러려면 이 `Sub`의 span이 어딘지, `Int`는 어디서 나왔는지 `String`은 어디서 나왔는지도 전부 밑줄을 쳐줘야함...
    - `ReturnType(Fn("foo"), (TypeVar(0), Int)) = TypeVar(1)`이 있는데 `foo`가 `Int`를 안 쓰는 경우
      - 그럼 `TypeVar(0)`이나 `TypeVar(1)`은 굳이 에러 낼 필요가 없음!
      - `Int`가 concrete type일 경우 (e.g. `let y = foo(x, 0)`), 나중에 type-check할 때 에러내는게 나음
      - `Int`가 infered type일 경우 (e.g. `let y = foo(x, z)`에서 `z: Int`라고 추론함), ... 애매하네
        - 다른 곳에서 실수가 있어서 `z: Int`라는 잘못된 추론이 나온건지, `foo`에다가 `z`를 넣은게 잘못된 건지 알 방법이 없음.
        - Rust는 이 경우 `z`에다가 밑줄 긋고, "expected String found Integer"라고 에러냄. 즉, `z: Int`라는 추론은 잘못이 없고 `foo`에다가 넣은게 잘못됐다는 입장!
2. 필요한 자료구조
  - `Map<def_span, Type>`: Concrete type이 나올 수도 있고, type var가 나올 수도 있음
    - func arg, let, name binding은 명확한데 func의 def_span을 넣으면 return type이 나와 아니면 전체 type (like `Fn(Int, Int) -> Int`)이 나와?
    - type var를 계산하면 이 map도 update해야함
    - 중간에 새로운 type var를 정의하는 경우도 있잖아? 이때는 def_span이 없음
      - `ReturnType(Op(Add), (_, Int))`에서 `_` 부분이 너무 길어서 새로운 type var를 정의한다고 치자.
      - def_span으로 검색할 일이 없을테니 이 map에 안 넣어도 되는 거 아님??
  - type var를 넣으면 걔랑 관련된 모든 type equation이 나오는 map
    - type equation이 계속 추가될 건데 그 과정에서 새로운 type var가 추가될 수도 있음
    - type var가 풀리면 이걸 통해서 모든 type equation을 update할 거임!
  - `ReturnType`과 관련된 처리
    - `Op(Concat)`를 넣으면 가능한 function signature를 전부 반환 (예: `Fn([T], [T]) -> [T]`, `Fn(String, String) -> String`)
      - 헉 generic function의 signature는 어떻게 적지??
      - 이걸 갖고 type equation 풀 거임. 예를 들어서 `ReturnType(Op(Concat), (String, TypeVar(0))) = TypeVar(1)`이 있으면 이 모양을 만족시키는 signature를 찾아서 `TypeVar(0) = String`, `TypeVar(1) = String`을 구할 수 있음!!
    - `Method("is_empty")`를 넣으면 얘가 정의된 type을 전부 반환 (예: `String`, `[T]`)
      - 여기서도 마찬가지... generic type의 signature를 어떻게 적지?
      - 이걸로 type inference하는 건 좀 별로이지 않음? 예를 들어서 `x.foobar()`를 보고 "foobar라는 method가 정의된 type은 MyInt밖에 없으니까 x는 MyInt구나"라고 하는 건 좀 이상한데 ㅠㅠ
        - 사용자 편의성 입장에선 오히려 좋은 건가?? 흠...
    - `Fn("map")`을 넣으면 걔의 function signature를 반환: 이거는 0개이거나 (오류) 1개이어야 함
      - 이거는 type inference 할 때 아주 요긴하게 씀!!
      - `ReturnType(Fn("foo"), (TypeVar(0), TypeVar(1))) = TypeVar(2)`가 있으면 `foo`의 signature만 보고 3개 다 풀 수 있으니까!!
3. 병렬 and/or on-disk 자료구조
  - type infer를 per-file로 할 수 있음? 이게 되면 병렬처리가 가능
  - 자료구조들을 싹다 memory에 올리는게 항상 가능할까?
4. pitfalls
  - Rust 문서 읽다가 알아낸 거: `TypeVar(0) = Option(TypeVar(0))`을 unify하려고 시도하면 무한 루프에 빠질 수도 있음. 이거 안 걸리게 조심하기!!

# 8. Linear type system

hir에서 count를 했으니 0번/1번/여러번은 이미 구분이 되잖아? 이거 hir에 표시해두자!

한 block에서, 각 name에 대해서

1. 몇번 쓰였는지 확인
  - 0번, (무조건) 1번, (조건부) 1번, (무조건) 여러번, (조건부) 여러번
2. 확인하면 뭐함?
  - 0번: warning, 정의 삭제
    - inline block은 unused_name이라는 개념이 되게 명확한데, top-level은 unused_name이 뭔지 애매함...
      - main에서 안 쓰면 unused인가? 근데 sodigy에도 bin/lib 구분이 있음? 있어야 할 거 같은데...
  - (조건부든 아니든) 1번: inlining
  - 여러번: ... 뭐 하지?
  - (무조건) 1번 이상: eager-evaluation 하는게 성능에 더 도움됨!
3. block에서 let을 다 없애는데 성공했으면 expr로 줄일 수 있음!
4. lazily-evaluated value를 최대한 줄이는게 목표!

생각해보니 이 작업은 mir에서 해야함!

1. function inline을 한 다음에 unused name 없애는 작업을 또 해야할 수도 있음. 근데 function inline은 mir 이후에만 가능!

function arg 갖고도 unused name warning을 해야함! 이거랑 같이 해? 따로 해? 하는 김에 같이 하는게 낫지 않나?

더 좋지만 복잡한 idea: unused function arg도 걍 삭제해버리면 되거든? 걔를 삭제하고 나서 block name counting을 하면 더 효율적일 수도 있음. 근데 func arg를 삭제한 다음에 block name counting을 하면 unused name warning이 헷갈릴 수 있음 (사용자 입장에선 used처럼 보이는데 unused로 셀 수도 있으니...)

conditional/unconditional 세는게 생각보다 빡셈

1. 어떤 block A의 conditional value X와 unconditional value Y가 있다고 하자
2. X 안에 있는 block B에 대해서,
  - B에서 정의한 let: 얘가 unused가 될지 아직 모르기 때문에 분석 불가. B의 value가 얘를 conditional 하게 호출하는지도 확인해야함! 만약에 B의 value가 얘를 conditional 하게도 호출하고 unconditional하게도 호출하면 어떻게 세야함??
  - B의 value에 있는 conditional value:
  - B의 value에 있는 unconditional value:
3. Y 안에 있는 block C에 대해서,
  - C에서 정의한 let: 얘가 unused가 될지 아직 모르기 때문에 분석 불가. C의 value가 얘를 conditional 하게 호출하는지도 확인해야함! 만약에 C의 value가 얘를 conditional 하게도 호출하고 unconditional하게도 호출하면 어떻게 세야함??
  - C의 value에 있는 conditional value:
  - C의 value에 있는 unconditional value:

# 6. Generics

돌고돌아서 결론

1. Rust notation을 따라하자: 똑똑한 사람들이 이렇게 만든 데에는 다 이유가 있다...
2. definition: `fn first<T>(ns: [T]): T = ns[0];`, `enum Option<T> = { None, Some(T) }`
3. annotation: `let ns: [Int]`, `let m: Map<String, [Int]>`
4. call: `collect::<Map<_, [_]>>()`
  - `_` notation도 일단은 허용?? ㅇㅇ 그러자
  - 걍 `::<` 자체를 하나의 operator로 묶어버릴까?
  - Rust는 path operator가 `::`이니까 turbo fish가 `::<_>`인게 말이 되는데, Sodigy에서는 `.<_>`로 하는게 맞지 않음??

Angle bracket 다루는게 불편하겠지만 어쩔 수 없음! 일단은 turbo fish operator가 있으니까 어찌저찌 구현은 될 듯?

# 4. Keyword Arguments

현재 구현

1. 함수 정의: default value 사용 가능. 다만, default value를 쓰기 시작하면 그 뒤의 모든 arg에 전부 default value를 붙여야 함 (Python과 동일)
2. 함수 호출: keyword arg 사용 가능. 다만, keyword arg를 쓰기 시작하면 그 뒤의 모든 arg에 전부 keyword를 붙여야 함 (Python과 동일)
  - positional arg 먼저 다 처리하고, keyword arg 다 처리하고, 그 다음에 남은 arg 중에서 default value 있는 애들 넣고, 그래도 처리 못하는 arg 있으면 error 던짐
3. functor: default value도 없고 keyword arg도 없음.
  - compile time에 파악 불가능한 함수에 keyword arg를 쓰면 무조건 error
4. function to functor
  - `Fn<(Int, Int): Int>` 자리에 `fn foo(x: Int, y: Int, z: Int = 5): Int`를 넣는 경우, `\(x: Int, y: Int): Int => foo(x, y, 5)`로 자동으로 바꾸기...??

1, 2, 3은 구현했고 4는 아직 미정
