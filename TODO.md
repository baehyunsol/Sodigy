# 7. name analysis

어떤 함수 안의 identifier X에 대해서, X는 1) 함수의 arg이거나 2) arg는 아니지만 함수 내부에서 선언된 값이거나 3) 둘다 아니거나. -> 딱 이것만 구분하면 됨!

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

# 5. Lambda function

문법을 좀 더 생각해보자.

1. parsing 용이성
  - `parse_func_arg_defs`를 그대로 쓰고 싶은데, 그러려면 arg와 expr이 확실히 구분되는게 편함!
2. 간결함: 너무 복잡하면 lambda의 의미가 없지!
3. 확장 가능성: 대부분 type annotation을 안 넣겠지만, 필요할 수도 있음!

- `\{x, y, x + y}` -> 현재 형태. default value 넣는 거나 type annotation 넣는 거나 전부 가능은 함!
- `\{$0 + $1}` -> arg를 아예 안 받는 형태! 극단적 간결함이 있지만, type annotation 넣는게 불가능해짐...
- `(a, b) => a + b` -> javascript style
- `lambda a, b: a + b` -> Python style
- `|a, b| a + b` -> Rust style
- `[](int a, int b) { return a + b }` -> C++ style
  - capture할 값들을 전부 입력해줘야함!
  - 아무리 못해도 이거보단 나을 듯?
- `\(a, b) => a + b` -> 이건 ㅇㄸ?
  - 괄호 안에다가 `parse_func_arg_defs` 그대로 쓸 수 있음
  - 뒷부분 parsing도 쉬움! ambiguity는 programmer가 parenthesis로 해결할 문제!!

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
