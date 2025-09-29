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

# 9. Type checks and inferences

1. inference를 먼저 하고 check를 해야겠네?
2. inference나 check를 하려면 `mir::Type`이 필요함. 근데 `mir::Type`을 만드려면 inference가 필요한 거 아님??
3. type check/inference는 inter-file로 해야함. 근데 지금 mir은 per-file로 할 계획이잖아? 그럼 안되지 않음..??
  - 하려면, inter-file hir을 만들면서 type check/inference에 필요한 정보를 미리 다 모아두고, mir은 per-file로 해야함.

일단 type을 어떻게 구현할지부터 정해야함!

1. first-class object
  - 완전 expr처럼 다루는 거임!
2. compiler built-in
  - 이건 좀 애매... custom struct도 처리해야하잖아?
3. 아니면... mir 끝난 다음에 type 처리해도 되는 거 아님??

# 8. Linear type system

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

1. funcs
2. structs
3. enums

일단 하지마! 단, built-in generic은 있음 (`List(Int)`, `Option(String)`, `Result(String, Error)`, `Map(String, List(String))`)

나중에 추가할 가능성이 있을까?

```
struct GenericSomething(T) = {
    generic_field: T,
    integer_field: Int,
    string_field: String,
};
```

나중에 이렇게 수정하려면 많이 복잡할까?
일단, angle bracket은 안 쓰고 싶음. 걔네는 group으로 안 잡혀서 parsing이 빡셈 ㅠㅠ

# 4. Keyword Arguments

Keyword arguments are necessary, especially if I want a declarative language.

1. Default values
  - Syntax is straigtforward: `func foo(x: Int = 3, y: Int = 4): Int = x + y;`
  - Since sodigy is purely functional, we don't have to worry about values and references like Python.
2. Mixing keyword arguments and positional arguments

func에다가 이걸 할 거면 struct field에도 default value 되게 하자! default value가 되면 type annotation이 optional해짐!!

1. 함수 정의할 때, default value 주기 시작하면 그 뒤로 전부 다 줘야함!
2. 함수 호출할 때, keyword arg 쓰기 시작하면 그 뒤로 전부 다 keyword arg여야함!

근데 이러면... type 검사 엄청 빡세지지 않음??

|   call \ definition    |        static (A)      | dynamic, but trackable (B)  |    dynamic (C)    |
|------------------------|------------------------|-----------------------------|-------------------|
| none                   |
| only positional        |
| only keywords          |
| positional + keywords  |

- A: `func`로 정의됐고, 이름 그대로 호출해서 추적 가능
- B: `\()`로 정의됐지만 정의와 호출이 바로 붙어있어서 쉽게 추적 가능
- C: `x: Func((Int, Int), Int)`로 한 다음에 `x()`를 한 경우

---

결정을 해야함

1. 아주 명확한 경우에만 keyword arg & default value를 허용하고 (위 표에서 A), 나머지는 전부 type error 처리
2. keyword arg와 default value까지 처리할 수 있는 type system을 만들기!!
  - `x: Func((x: Int = 3, y: Int = 4), Int)`로 하면... 아 저렇게 쓰면 parsing 불가능 ㅠㅠ
  - `x: Func((Arg(name="x", type=Int, default=3), Arg(name="y", type=Int, default=4)), Int)` 이러면 되긴 함ㅋㅋㅋ
    - 와 저거 어떻게 구현하냐...

---

1. function이 됐든 lambda가 됐든 comp-time에 정의를 찾을 수 있는 경우
  - 어떻게든 찾아서 keyword arg랑 default value 적용하기
2. 정의를 찾는게 불가능하고 `f: Func((Int, Int), Int)`의 정보만 있는 경우
  - `f()`를 할 때는 keyword arg 허용 안하고 무조건 `(Int, Int)`를 기대함 (당연히 default value 같은 것도 없음)
  - `f = foo`를 할 때는 `foo`를 최대한 `Func((Int, Int), Int)`에 맞추기
    - 예를 들어서 `func foo(x: Int, y: Int, z: Int = 5)`를 `f`에 집어넣으면 `z=5`로 고정해버리면 되지? 그럼 고정하는 거지...

ㅋㅋㅋ 너무 복잡한데?

# 3. DocComments and Decorators

1.

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
