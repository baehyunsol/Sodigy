# 57. `mod` and `use`

1. `mod` 없어도 되지 않음?
  - `use`에서 처음보는 이름이 있으면 `mod`라고 지레짐작하고 가져오는 거임!
  - rust에서는 inline `mod`도 가능함: `mod tests { ... }` -> 이거 우리도 됨?
    - 구현이 trivial하면 되게 하자
2. `use` 문법
  - 최대한 rust랑 비슷하게하기...는 별 의미가 없음. 우리는 `::`대신 `.`를 쓰기 때문
  - 그럼 `::`를 쓸까? 이건 아예 별개의 issue이고 난 살짝 반대임
  - `use x.y;` is equal to `use x.y as y;`
  - `use x.{y, z as zz, y.k, y.l as ll};` is equal to `use x.y as y; use x.z as zz; use x.y.k as k; use x.y.l as ll;`
3. 파일구조
  - module의 구조는 rust랑 동일하게 하자 (`foo.rs` 혹은 `foo/mod.rs`).
    - 사실 `mod.rs`말고 더 좋은 이름 있을 듯? 이건 생각해보자.
    - `mod.rs`는 rust에서 `mod`가 keyword이기 때문에 가능함. 나도 keyword를 쓰거나 잘 회피를 하거나 해야함...
      - 아 이래서 `mod`를 쓰는 건가? ㅋㅋㅋ
      - 그럼 rust에서 `mod r#mod`를 하면 어떻게 되나... 해봤더니 circular module error가 뜸! 이것까지 대비를 해뒀구나...
  - rust는 module 위에 crate가 있음. 우리도 이런 거 있으면 좋은데...
4. re-export
  - rust에서는 `pub use`로 re-export가 가능함. 우리도 됨?
  - 구현이 쉬울 거 같은데, 되게하자!

# 56. byte/bit pattern matching

- some drafts
  - `1010xxxx`: 8 bit integer that is in range `160..=175`. The matched integer is in range `0..=15`
  - `1010..xxxx`: an arbitrary size integer that starts with `1010`. It matches the last 4 bit of the integer.
  - No... not this way. It's too confusing.

# 55. `r#keyword` -> implement this in lexer

# 54. KeywordArgumentRepeated error

if `x` is repeated 3 times (let's say x1, x2 and x3). it throws an error for x1&x2, and x2&x3. I want it to merge those errors...

# 53. `pub` keyword

Same as Rust

Attribute parser can handle this. Make sure to break immediately after parsing `pub` keyword!

1. `pub`, `pub(crate)`, `pub(mod)`
  - Are we gonna use the term `crate`?
  - IIRC, `mod` is already a keyword. Should `pub` and `crate` also be keywords?
2. `pub(self)`, `pub(super)` -> I haven't seen these or used these
3. `pub(in crate::tools)` -> what is this?

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

# 49. even more on type system

1. 생각해보니까 subtyping이 필요함... never type이 있거든 ㅠㅠ
  - 일단은 strict typing만 구현해두고 나중에 생각 ㅠㅠ
2. 이제 type expression이 필요해졌음
  - `fn add(x, y)`가 있으면 `add`의 def_span으로 검색을 하면 `Fn(TypeVar(x), TypeVar(y)) -> TypeVar(add)`가 나오겠지? 만약에 `TypeVar(x)`를 풀었다고 치자. 그럼 `TypeVar(add)`도 검색을 해서 여기도 반영을 해줘야하는데 그게 안되고 있음
  - 풀다 보니까 `TypeVar(a) = TypeVar(b)`라는 equation을 만들었다고 치자. 지금 구현으로는 이걸 처리할 방법이 없음!
  - As of now, lhs must be a simple TypeVar, because all we can do now is to get a type expression from `types.get(x)`.
    - Let' create `type_var_rev_map: HashMap<Span, Vec<Span>>`.
    - In the above case, it looks like `{ x: [add], y: [add] }`, because `TypeVar(x)` is referenced by `TypeVar(add)` and `TypeVar(y)` is also referenced by `TypeVar(add)`.
    - When we solve `TypeVar(x)`, we iterate type vars that are referencing `TypeVar(x)`, and replace `TypeVar(x)` with the result.
  - In the future, we might have more complex type equations, where both sides are type expressions.
    - We can solve this with `type_var_rev_map`.
    - Think `Result<TypeVar(x), TypeVar(y)> = Result<TypeVar(z), TypeVar(w)>`.
    - We first create intermediate type vars: `TypeVar(i1) = Result<TypeVar(x), TypeVar(y)>` and `TypeVar(i2) = Result<TypeVar(z), TypeVar(w)>`.
    - We also add a type equation `TypeVar(i1) = TypeVar(i2)`.
    - When we solve `TypeVar(x)`, we're gonna also update `TypeVar(i1)`.
    - When we solve `TypeVar(i1)`, we're gonna update `TypeVar(i2)`.
3. compiler debugging 하기 좋은 아이디어!!
  - 이제 type infer 결과물도 다 있고, span도 다 있고, visualize도 다 되고, 심지어 누가 infer됐는지도 다 있음.
  - 그럼 infer된 결과물 다 보여주는 CLI command를 만들까?

# 48. Compiler & Sodigy std

Compiler가 Sodigy std를 직접 참조해야할 일이 아주 많음

```sodigy
// Sodigy std

// There's no implementation because it's a built-in type.
type Int;

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
  - 이거는 자동으로 할 방법이 없음. `Int`에다가 `@lang_item("Int")`라고 붙여주고 type-infer engine이 `lang_item("Int")`를 사용하면 됨.
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

# 38. More on memory

1. dec_rc를 한 다음에 destructor를 호출하려면... 현재 보고 있는 값이 Integer/String인지 Compound인지 알아야 함!
  - 만약 Compound라면 element는 몇개인지, 각 element의 type은 뭔지도 알아야 함...
2. ref-count 분석을 했다고 치자... 그래서 뭘 할 수 있지? 어차피 다 heap에 올라가면 이득이 거의 없는 거 아님??
  - 즉, heap-allocation을 아예 피하는 방법을 찾아야함, how?
3. in-place mutation -> bytecode로 어떻게 표현? ref_count는??
  - 지금은 그냥 `Register::Call(0)`에다가 struct 두고 `Register::Call(1)`에다가 index 두고 `Register::Call(2)`에다가 value 둔 다음에 `Intrinsic::Update` 해야겠지?
  - 그럼 `Intrinsic::Update`가 새로운 struct 만들고, 기존 field 복사하고 (update할 field 빼고) (이때 inc_rc도 하고), value도 복사하고 (이때 inc_rc)도 하고, 이거 끝나면 `Register::Call(_)`에 있는 값들 pop하면서 dec_rc도 함.
  - in-place로 하려면 새로운 struct 만드는대신 기존 struct를 inc_rc 하고, 기존 field는 건드리지 말고, value는 복사해서 inc_rc하고, 덮어씌워지는 값은 dec_rc 하고, 그럼 됨!
    - 조금 더 싸네
  - 근데, Sodigy에 cyclic reference가 없는 이유가 in-place mutation이 없기 때문이잖아, 이 최적화를 하면 cycle이 생길 수도 있는 거 아님??
    - 생각해보니까 될 듯... 이거 어케 막지? type 보고 cycle이 가능한 경우에는 in-place mutation을 꺼야 함!

Canceling reference count isn't a big deal. The big deals are

1. In-place mutations.
2. Turning `match (x, y, z) { (0, 1, 2) => _ }` into `let e1 = x; let e2 = y; let e3 = z; if x == 0 && y == 1 && z == 2 { _ }`.

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

일단은 보류하고 (아직은 debugging이 필요할 정도로 긴 Sodigy 코드를 못 짬), Sodigy 코드를 많이 짜고 나서 그때 생각할까?

# 36. Impure IO

지금 생각한 거는,

```
fn main(world: World) -> World = match foo() {
    $whatever => main(
        world
            .print("Hello, World!")
            .write_string("file.txt", "Hello, World!")
    ),
    _ => world.quit(),
};
```

이런 식으로 하는 거임. 모든 impure function은 `world`를 통해서만 호출 가능. `main`에서 나가는 순간 `world`에 붙어있는 impure action을 다 처리함. `main`을 recursive하게 호출함으로써 impure action의 결과를 사용할 수도 있음. `World`는 `main`에서만 사용 가능.

근데 이러면 action의 결과를 어떻게 읽어?

What if we want a library of impure functions?

---

Some notes

1. In rust, it's okay to call main functions in other functions!!
2. In Sodigy, what if someone tries `fn main(world: World) -> World = { let sim1 = main(world.do_something1()); let sim2 = main(world.do_something2()); if world.cond() { sim1 } else { sim2 } }`?
  - In order to prevent this, `world` and `world.do_something()` must be different types...

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
    - How about adding `@unused` decorator?
      - Just being curious here,,, is it okay to use a name that's decorated with `@unused`?
      - How about `@allow(unused)`?
        - well... currently the parser uses expr_parser to parse the arguments of a decorator. But the hir's expr_parser won't allow the identifier `unused`. There are a few ways to fix this:
        - First, we can implement a separate parser for decorators. But then we have to write parser for each decorator. That'd be huge!
          - Hir has to do this. If we choose a right timing, it can access to defined names (if it has to), and use undefined names (if it wants to).
        - Second, we can add `unused` to namespace (only when parsing decorators).
        - Third, we can use `@allow("unused")` syntax.
  - If a name is used by expressions only once, and multiple time by assertions, we inline the name anyway. For example, `{ let x = foo() + 1; assert x > 0; assert x > 1; [x] }` becomes `{ let x = foo() + 1; assert x > 0; assert x > 1; [foo() + 1] }`.
    - We need a lot of tweaks here...
    - `let x` statement is removed in release mode, but not in debug mode.
4. Assertions that are enabled in release mode.
  - How about `@always` decorator?
  - The compiler treats such assertions like an expression, not an assertion.
  - If a top-level assertion is decorated with `@always`, it's asserted before entering the main function.
    - It's ignored in test-context.
5. Syntactic sugar for `assert x == y;`
  - 이게 실패하면 lhs와 rhs를 확인해야함...
  - 근데 syntax 기준으로 뜯어내는 거는 너무 더러운데... ㅜㅜ 이건 마치 `==`를 syntactic sugar로 쓰겠다는 발상이잖아 ㅋㅋㅋ
  - 아니면 좀 덜 sugarly하게 할까? 그냥 모든 expr에 대해서 다 inspect 하는 거임 ㅋㅋㅋ
    - value가 `Call { func: Callable, args: Vec<Expr> }`인 경우, `func`랑 `args`를 dump (infix_op도 다 여기에 잡힘)
    - value가 `Block { lets: Vec<Let>, value: Expr }`인 경우, `lets`를 dump (expr만), `value`는 dump할 필요없음 (당연히 False일테니)
    - value가 `if { cond: Expr, .. }`인 경우, `cond`를 dump, `value`는 dump할 필요없음 (당연히 False일테니)
    - value가 `match { value: Expr, .. }`인 경우, `value`를 dump하고 어느 branch에 걸렸는지도 dump
6. Naming assertions: `@name("fibo_assert")`.
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

# 27. 개발 방향

1. embedding language, interpreter 전부 고려 X. Cargo스러운 compiler만 개발
  - 즉, 중간 파일을 많이 만들어도 상관없고, 프로세스를 많이 띄워도 상관없음.
2. FFI: 일단은 고려안함. 모든 코드는 Sodigy로만 작성됐다고 가정
3. 메모리 최적화 기준: 4GiB

# 25. Make it more Rust-like!! ... 하다가 생긴 문제점

Name binding에 `$`를 안 붙이니까 한가지 문제가 생김: `True`랑 `False`에 match 하려면 `$True`, `$False`를 해야함... Rust는 `true`/`false`가 keyword여서 이런 문제가 없음.

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
