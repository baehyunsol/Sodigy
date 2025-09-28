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
