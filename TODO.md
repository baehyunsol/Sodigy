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

# 22. Type annotation 추가 수정

1. Functors: `Fn<(u32): u32>`
  - function에서도 return type을 `:`로 쓰니까 여기서도 `:` 쓰자!
  - 다른 곳에서도 angle bracket 쓰니까 여기서도 angle bracket 쓰자
  - `(u32)`에는 special parser를 쓸 거기 때문에 trailing comma 없어도 됨!
2. list: `[T]`

# 21. Type annotation 생략

모든 type annotation은 생략 가능. 단, type-infer가 불가능하면 무조건 compile error

심지어 `fn first<T>(l: [T]): T = l[0];`에서도 생략 가능함. 그대신 이러면 `T`라는 generic도 못씀 (generic이 정의되면 반드시 1번 이상 쓰여야 하거든). 이러면 나중에 type-infer 할 때나 type-check 할 때 error가 나겠지...

# 20. test bench

지금은 `main` 만들어서 일일이 실행하고 있지? 얘네를 얼른 `@test`로 바꿔야함!

```
@test.eq((10, 20), 30)
fn adder(x, y) = x + y;

@test.eq(50)
let value = adder(20, 30);
```

becomes

```
// for all elements, it prints the message to stderr if the result is False
let assertions: List((Bool, String)) = [
    {
        let arg = (10, 20);
        // 여기가 문제: `._0`과 `._1`이 있다는 걸 알려면 `arg: (Int, Int)`라는 사실을 알아내야함...
        // 나머지는 단순 ast 조작으로 구현 가능!
        // 아니면... compiler built-in function을 만들어서 `call(adder, arg)`처럼 쓸까? 흠...
        let result = adder(arg._0, arg._1);
        let expected = 30;

        // 함수 이름이나 span도 적어주면 더 좋을 듯?
        (result == expected, f"Input: {arg:?}, Expected: {expected:?}, Got: {result:?}")
    }, {
        let result = value;
        let expected = 50;

        // 상수 이름이나 span도 적어주면 더 좋을 듯?
        (result == expected, f"Expected: {expected:?}, Got: {result:?}")
    },
];
```

# 19. cycle-checks in `let` values

```sodigy
// 이건 당연히 안됨! cycle-checker가 걸러내야함
let x = y;
let y = x;

// 이건 되어야 하는데... 구현이 쉽지 않음 ㅠㅠ
// 그냥 하지말라고 할까??
let f1 = \(x) => if x < 2 { 1 } else { f2(x - 1) + f2(x - 2) };
let f2 = \(x) => if x < 2 { 1 } else { f1(x - 1) + f1(x - 2) };

// f1, f2랑 동일한 구조인데 얘는 됨.
fn f3(x) = if x < 2 { 1 } else { f4(x - 1) + f4(x - 2) };
fn f4(x) = if x < 2 { 1 } else { f3(x - 1) + f3(x - 2) };

// 조금 더 뇌절을 한 버전, 따지고 보면 얘네는 closure가 아니거든? 근데 closure가 아니라는 걸 알기가 쉽지 않음...
let f5 = {
    let ONE = 1;
    let TWO = 2;

    \(x) => if x < TWO { ONE } else { f6(x - ONE) + f6(x - TWO) }
};
let f6 = {
    let ONE = 1;
    let TWO = 2;

    \(x) => if x < TWO { ONE } else { f5(x - ONE) + f5(x - TWO) }
};
```

아...

# 18. negative index

`a[-1]`을 하면 맨 마지막 element를 주기

1. a에 element가 20개인데 `a[-200]`를 하면 10바퀴 돌아? 아니면 `[-20]` 밑으로는 다 error?
2. `a[2..10]`은 slice로 할 거잖아, 그럼 `a[2..-1]`도 돼?
  - 근데 `2..-1`은 그자체로 runtime error 아냐? 아닌가...
  - Rust에서 `.get(10..2)`로 하니까 `None` 나옴. 즉, `10..2` 자체는 문제가 없음!

# 17. (un)intern_string

intern은 context-free여서 상관이 없음...
unintern을 하려면 `HashMap<InternedString, String>`이 있어야함!

1. 이거를 Session이 계속 들고 있어? 그건 좀 비효율적...
2. 이거를 파일에 적어두고 그때그때 꺼내서 쓰기?
  - 파일 위치를 session들한테 알려줘야 하는데? config스러운 거를 만들어서 계속 들고다녀야할 듯?
  - 병렬로 돌아갈테니 lock 걸어야 할 듯?
  - 근데, string이 무지무지하게 길 수도 있잖아? 그럼 또 나눠??
    - 0..=15 글자: InternedString에 바로 저장
    - 16..=4095 글자: 파일에 바로 저장
    - 4096.. 글자: 파일에는 포인터만 저장되고 딴 파일을 열면 거기에 들어있음 ㅋㅋㅋ
    - ㅋㅋㅋㅋ 이거 너무 뇌절 아님??

# 16. span across files

지금은 single file이니까 상관이 없지만, span 안에서 각 파일을 나타낼 방법을 좀 더 고민해봐야함!

1. 한번에 여러 파일을 컴파일하는 경우
  - 여러 파일을 전부 cli로 넣어줘? 아니면 `mod` 보고 얘가 알아서 찾아?
  - 
2. incremental compilation을 하면 hir을 저장해야함. 그때 hir의 span도 저장될텐데, ...
3. package manager를 만든다고 치면, hir은 컴파일된 상태로 배포를 할 거지? 그럼 이 안에 있는 span은 어떻게 하려구...

# 15. name bindings and decorators

Name bindings in patterns use `@` character, and so do decorators. But the problem is that the lexer treats `(b'@', b'a'..=b'z')` as a single token (decorator token). So the pattern parser has to be aware of this.

# 14. error vs warning

지금 `struct Error`랑 `struct Warning`이랑 너무 비슷한데 차라리 하나로 통일하고 error인지 warning인지 구분하는 flag만 추가할까...

# 13. prelude

어느 시점에 집어넣어야 하나...

1. hir에서 `NameOrigin` 찾는 시점에 이미 있어야 함
  - Namespace 맨 위에 넣어주고 시작하면 됨!
  - span은 `Span::Prelude`로 주자!
2. mir에서도 `Span::Prelude` 보고 걔의 shape를 알 수 있어야 함!
  - MirSession에다가 `Map<Span, Shape>` 넣어줘야 함!

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
// - `TypeVar(0) = Fn((TypeVar(3),), TypeVar(4))`
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
let x = if pat Some($n) = foo() { bar(n) } else { baz };
let y = x + 1;

// 1. type annotation이 있어야하는 자리에 type annotation이 없으면 추가하고 시작
// - `foo: TypeVar(0)`
// - `x: TypeVar(1)`
// - `y: TypeVar(2)`
// - `foo_ret: TypeVar(3)`
// - `n: TypeVar(4)`
//
// 2. `let foo`의 좌변과 우변을 비교해서 추론
// - `TypeVar(0) = Fn((), TypeVar(3))`
//
// 3. `let x`의 우변에 있는 if문 뜯기, 먼저 cond부터
// - `TypeVar(4) = Option(TypeVar(4))`
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

fn foo<T, U>(a: T, b: U): T = a;
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

fn first<T, U>(ls: [T], b: U): T = ls[0];
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
    - func arg, let, name binding은 명확한데 func의 def_span을 넣으면 return type이 나와 아니면 전체 type (like `Fn<(Int, Int): Int>`)이 나와?
    - type var를 계산하면 이 map도 update해야함
    - 중간에 새로운 type var를 정의하는 경우도 있잖아? 이때는 def_span이 없음
      - `ReturnType(Op(Add), (_, Int))`에서 `_` 부분이 너무 길어서 새로운 type var를 정의한다고 치자.
      - def_span으로 검색할 일이 없을테니 이 map에 안 넣어도 되는 거 아님??
  - type var를 넣으면 걔랑 관련된 모든 type equation이 나오는 map
    - type equation이 계속 추가될 건데 그 과정에서 새로운 type var가 추가될 수도 있음
    - type var가 풀리면 이걸 통해서 모든 type equation을 update할 거임!
  - `ReturnType`과 관련된 처리
    - `Op(Concat)`를 넣으면 가능한 function signature를 전부 반환 (예: `Fn<([T], [T]): [T]>`, `Fn<(String, String): String>`)
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

# 11. Byte Code (Or LIR)

### 1. block

```sodigy
{
    let eager = foo(3, 4);
    let lazy = bar(3, 4);

    // this is tail call
    eager + lazy
}
```

```c
// uninitialized state of `lazy`
local1.push(nullptr);

// eval `eager`
r1.push(3);
r2.push(4);
call_stack.push(s1);
goto foo;
label: s1
call_stack.pop();
r1.pop();
r2.pop();
r1.push(ret);

// eval `lazy`, if it has to
jump_if_init(local1, s2);
r1.push(3);
r2.push(4);
call_stack.push(s3);
goto bar;
label: s3
call_stack.pop();
r1.pop();
r2.pop();
local1.assign(ret);

label: s2
r2.push(local1);

local1.pop();
// this doesn't push to call_stack because it's a tail call
goto add;
```

### 2. if

```sodigy
// `x` and `y` are at `r3` and `r4`
// this `if` is tail-call
if foo(x, y) { bar(3, 4) } else { baz }
```

```c
r1.push(r3);
r2.push(r4);
call_stack.push(s1);
goto foo;
label: s1
call_stack.pop();
r1.pop();
r2.pop();
r1.push(ret);

branch(r1, s2, s3);
label: s2
r1.pop();
r1.push(3);
r2.push(4);
call_stack.push(s4);
goto bar;
label: s4
call_stack.pop();
r1.pop();
r2.pop();
goto call_stack.peek();

label: s3
r1.pop();
ret.push(baz);
goto call_stack.peek();
```

### 3. if, with assignment

```sodigy
// This is a tail-call
if pat Some($x) = foo(3, 4) { bar(x) } else { baz };
```

```c
// place for `x`
local1.push(nullptr);

r1.push(3);
r2.push(4);
call_stack.push(s1);
goto foo;
label: s1
call_stack.pop();
r1.pop();
r2.pop();
local1.assign(ret);

r1.push(local1);
call_stack.push(s2);
goto is_some;
label: s2
call_stack.pop();
r1.pop();
r1.push(ret);

branch(r1, s3, s4);
label: s3
r1.pop();
r1.push(local1);
local1.pop();
goto bar;  // this is a tail call

label: s4
r1.push(baz);
local1.pop();
goto call_stack.peek();
```

# 10. func arg errors

1. positional arg만 있는 경우
  - expected 5, got 4
    - 뭐가 missing인지 찾을 수 있음??
    - default value가 있으면 머리 아픔...
  - expected 5, got 6
  - expected 5, got 5, but there's a type error

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

# 1. Complete Rewrite!!

Let's make it 1) simple enough that I can implement and 2) complicated enough that I don't get bored.

1. Type system
  - No type classes, no generics, and very simple type inference.
  - Type classes and generics are all, if exist, compiler-built-in.
2. Purely functional
3. Block-based
  - A block consists of zero or more declarations (scoped) and one expression.
  - A block is wrapped in curly braces.
  - A block is a valid expression.
  - Entire code-base is a block (curly braces are omitted). If expressions and match branches also use blocks.
