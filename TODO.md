# 123. ErrorKind가 안 떠올라서 막혀있는 상황들

1. inter-hir: `#[impl(std.op.add)] fn add_int()`
  - `std.op.add`가 `#[poly]`가 아닌 경우
  - `std.op.add` 자리에 다른 expr이 들어있는 경우
2. inter-hir: `type x<T> = _; use x.y.z as w;`
  - `x.y.z`에서 `y`에 밑줄치고 에러 날려야하는데 뭐라고 할지 생각이 안남...
3. mir: struct init에서 struct 자리에 ident 말고 다른게 들어있을 때
4. mir-type: `#[impl(std.op.add)] fn add_int()`
  - `std.op.add`의 type infer가 덜됐을 경우
  - `add_int`의 type infer가 덜됐을 경우
  - `std.op.add`와 `add_int`의 parameter 개수가 다른 경우
  - `std.op.add`와 `add_int`를 맞춰봤는데 type이 안 맞는 경우

# 122. Very long files

Bottlenecks: 1) lexer has to load the entire `Vec<u8>` of a file 2) parser/hir has to load the entire AST of a file 3) mir has to load the entire project 4) an `InternedString` can intern at most 2 billion bytes 5) the interpreter's memory allocator can allocate a block of at most 2 billion scalars.

4 and 5 are the most serious ones. The current implementation can do nothing if there's a string literal which is larger than 2 billion bytes.

It's easy to fix 4: we can use more bits for length and less bits for hash when the string is long.

Scenarios:

1. There's a very large string in a file.
  - If it can pass the bottleneck #1, #4 and #5, everything's good.
2. There's a very large object (e.g. a list with 4 billion integers) in a file.
  - Oh no...
3. There's a very long comment in a file.
  - If it can pass the bottlenect #1, everything's good.
4. Each function is small, but there are billions of functions in a file.
5. Each file is small, but there are millions of files in a project.
6. etc

---

아니 근데, 애초에 scalar에 32bit를 쓰고 있는데, 그럼 4GiB 넘는 string은 절대 못 쓰는 거 아님?? scalar를 가변으로 쓰지 않는 이상...

생각해보니, heap이 쓰는 메모리가 32bit 영역을 넘어가면 런타임 에러를 던져야함 -> 지금은 이런 검사가 전혀 없음!!
-> `Heap::expand()`가 하면 됨!

추가로, string literal이 4GiB 넘어가면 compile error를 날려야겠네?
하는 김에 decimal digit이 너무 길어도 warning 날리자

# 121. Pipeline operator

```
a() |> b($, 1) |> $ + 1;
// ->
{
  let t1 = a();
  let t2 = b(t1, 1);
  t2 + 1
};

a() |> 1 + b($) + (c() |> d($)) |> e($);
// ->
{
  let t1 = a();
  let t2 = 1 + b(t1) + { let t1 = c(); d(t1) };
  e(t2)
}
```

1. gleam이나 bash처럼 implicit하게 넘기는 건 별로고, `$`로 직접 넘기자!!
2. 기존의 expr variants 재활용하지 말고, 아예 새로운 variant를 만들어버리자!!
  - `|>`의 precedence를 제일 아래로 내려버리면 parsing이 쉬워짐, 일단 pratt_parsing한 다음에 rhs만 쭉 펼치면 되거든
  - 그런 다음에 `Pipeline(Vec<Expr>)`로 만들어버리자!!
3. unused name은 언제 잡을까?
  - hir에서 잡는 것보다는 parser가 dedicated error variant 날리는게 나을 듯?
  - unused인데 뒷부분에 identifier 하나만 덩그러니 있으면 error note에다가 "perhaps you mean f($)?"라고 하자!!
  - 이건 warning 날리지말고 error 날려버리자!!

# 119. idea for testing the type-solver

원래 정상적으로 도는 프로그램이 있을 때, 그 프로그램 안에 있는 type annotation을 삭제한 다음 돌리면 type-error (cannot-infer)가 나거나 정상적으로 돌거나 둘 중에 하나이어야함!!

compiler가 type annotation을 무시하도록 구현해야할 듯??

1. type annotation을 다 지우고 돌리기
2. 랜덤으로 일부만 지우고 돌리기

근데 assertion이 있으면 type-infer가 너무 쉬운데...

아니면, assertion도 지우고 type annotation도 지운 다음에 결과물의 type을 직접 비교할 수도 있음 (어차피 span은 다 똑같으니까 type이 완전히 동일해야함) -> (type annotation을 실제로 삭제하는게 아니고 숨기는 거여서 span은 변하면 안됨)

아니면, type annotation을 잘못 준 다음에 (정답이 `String`인 걸 알 때 강제로 `Int`를 집어넣음), 오류가 나는지 확인해도 되고

# 118. un-static top-level let

top-level let인데 덩치가 너무 커서 static으로 만들기는 부담스러운 경우...

1. 특정 decorator를 붙이면 gc 되도록 관리?
2. 구현은 쉬움 `let x = foo();`를 `let x = x_eval(); fn x_eval() = foo();`로 바꾸면 됨!!
  - 생각해보니까 이래도 결국은 결과물이 static 하게 남네? 걍 `let x`를 `fn x()`로 바꾼 다음에 `x`를 언급하는 모든 곳을 찾아서 `x`를 `x()`로 바꿔야함 ㅋㅋㅋ ㅠㅠ
  - 아니지 오히려 runtime에서 구현하면 훨씬 간단: initial reference count를 1로 주지 말고 0으로 주면 되네!!

# 117. shift vs type annotation

type annotation에서 `<<`나 `>>` 나오면 처리가 안됨 ㅠㅠ

1. `tokens::peek`을 했을 때 쪼개서 주기..?? 는 말도 안되고
2. 애초에 `<<`를 (`<`, `<`)로 쪼개서 저장해뒀다가 parser가 합치기?
  - 이러면 `<`들 사이에 띄어쓰기가 있을 때 대응이 안됨
  - 그럼 trailing_whitespace를 field로 추가하기..??

# 116. `error_span()`

`error_span_wide()`랑 `error_span_narrow()`로 구분해서 쓸까? narrow는 operator나 keyword만 해주는 거지!!

# 115. Span tester

text file이랑 span을 주고, render-span 호출할 수 있게하는 pipeline 만들자!

# 114. regex patterns

`r""`를 raw string으로도 쓰고 regex pattern으로도 쓰는 거는 너무 헷갈릴 듯??

차라리 `re""`를 만들어서 regex pattern으로 쓰자!!

pattern에서는 `re""`가 regex고... expression에서는??

1. `r""`이랑 동일하게 취급하기
2. 에러 날리기
3. `Regex`라는 type 새로 만들어서 그걸로 취급하기
  - regex library 사용해도 저 type이 나옴
  - regex literal은 반드시 compile time에 eval됨!

# 112. lists

이거 분명히 옛날에 issue 있었는데...

`a ++ b`, `a[i..j]`, `a[i]`, `a <+ x`, `x +> a`, `a.pop_front()`, `a.pop_back()`, `a.update(i, x), a.len()` 중에서 몇개를 O(n)으로 만들고 몇개를 O(1)으로 만들지를 결정해야함.

1. Rust vector 형식으로 저장
  - O(n), O(n), O(1), O(n), O(n), O(n), O(n), O(n), O(1)
2. Rust slice 형식으로 저장
  - O(n), O(1), O(1), O(n), O(n), O(1), O(1), O(n), O(1)
  - 1번에 비해서 모든 연산이 조금씩 느려짐 (time complexity가 동일할 때)
3. singly linked list 형식으로 저장
  - O(1), O(n), O(n), O(n), O(1), O(1), O(n), O(n), O(n)
  - 1번에 비해서 모든 연산이 조금씩 느려짐 (time complexity가 동일할 때)
  - 이렇게 하면 string이 너무 느려짐...

2번이 젤 나아보이긴 하지만, 어디까지 built-in으로 처리하고 어디부터 sodigy로 할지도 애매함. 예를 들어서, pattern matching에서 `[a] ++ r`이 있으면 저기 있는 `[a]`는 slice야? vector야?

아니면 2번을 아예 builtin으로 처리해버려?? ptr, start: scalar, end: scalar로 돼 있는 struct인데 runtime level에서 다 관리되는 거임. 이게 되려면 위에서 나열한 operation들 전부 builtin으로 구현해야함 ㅋㅋㅋ

# 111. more diverse `Span::None`

compiler가 새로운 token/expr을 만들어 낼 일이 아주 많음!!

1. 단순히 `Span::None`을 주면, uniqueness도 깨지고 error message도 표현 불가
2. parent expr의 span을 그대로 가져다가 쓰면, uniqueness가 깨짐
  - 기존 span의 uniqueness가 깨지기 때문에 이상한 error가 발생할 수 있음...

몇가지 예시: `a && b`를 `if a { b } else { False }`로 바꾸는 경우, `f"{a}, {b}"`를 `to_string(a) ++ ", " ++ to_string(b)`로 바꾸는 경우.

1. `to_string`과 `++`에 span을 unique하게 줘야 generic을 풀 수 있음.
2. `to_string(a)`에서 오류가 날 경우, span이 있어야 설명이 가능
  - 근데 어떻게 설명함??
  - `a`에다가 밑줄을 긋고 ... 뭐라고 알려주지? ㅋㅋㅋ
3. 하는 김에 generic monomorphization도 이걸로 해버려??
  - generic monomorphization은 여러 단계로 할 수 있기때문에 그거 고려해야함

# 110. lessen cyclic let detections

`let f1 = \(_) => _; let f2 = \(_) => _;`처럼 돼 있으면 `f1`이랑 `f2`랑 서로 언급하더라도 봐주자...

1. 이건 hir level의 단순한 휴리스틱임: rhs가 `Expr::Lambda`면 걍 봐주는 거임 ㅋㅋㅋ
  - 물론 이렇게 해도 못 잡는 예외가 많지만, 어쩔 수 없음!!
2. 한가지 예외가 있음... 만약 f1이나 f2가 default value로 서로룰 언급하면 cycle이 생길 수 있음
  - 이거는 다른 방식으로 막자. lambda는 default value를 선언하는 거 자체를 못하게 할 거임. 어차피 lambda에서는 default value가 의미가 없거든 (애초에 compiler 차원에서 추적이 불가능함.)

# 109. `JumpIfUninit`

결론:

1. top-level let은 lazy-eval, inline let은 eager-eval
  - 문서에다가 values might be lazy-evaled or eager-evaled라고 적어야 함
  - top-level let은 static임
2. top-level let이 init 됐는지 안 됐는지는 runtime에서 관리!

# 107. top-level let eval strategy

top-level let statements are lazy-evaluated and static (once evaluated, it lasts in memory forever).

The optimizer might choose to evaluate the value at compile time. What if I add a decorator that forces the value to be evaluated at compile time?

# 106. Sub-enums

어떤 함수가 `hir::Expr`을 반환함. 근데 얘가 항상 `hir::Expr::Number`나 `hir::Expr::String`만 반환하는 거야! 그러면 match를 해서 저 둘만 잡고 나머지는 전부 `unreachable!`을 때리겠지? 이거를 type-system 차원에서 할 방법이 없을까?

특정 variant만 갖는 enum을 sub-enum으로 정의하는 거임

https://gist.github.com/joboet/0cecbce925ee2ad1ee3e5520cec81e30

# 105. tmp name binding in patterns

```rs
enum Expr {
    Infix { op: Op, lhs: Option<Expr>, rhs: Option<Expr> },
    Postfix { op: Op, lhs: Option<Expr> },
}
```

가 있다고 치자. `Infix`하고 `Postfix`한테 아주 비슷한 작업을 동시에 하고 싶을 때가 있음. 그러면 `Postfix`한테 `rhs = None`을 주고 작업해버리면 됨... 근데 rust 문법에서는 이게 안됨!!

# 104. let-destructures

참고로 Rust에서는 `let (Ok(y) | Err(y)) = x;` 할 수 있음...!! 즉, type-check를 하고 나서 destructure를 함...

InfixOp를 허용하면 `let x + 1 = y + 1;`도 되겠네? ㅋㅋㅋ 말도 안돼...

내 원래 계획은, `let (x, y, _) = foo();`가 있으면, 얘를 `let tmp = foo(); let x = tmp._0; let y = tmp._1;`로 바꾸는 거였음. 이렇게 하면 문제가

1. 유저가 만들지 않은 코드를 에러메시지에서 언급하면 유저가 헷갈림.
2. type check, refutability check를 위해서는 mir이 끝날 때까지 원본을 들고 있어야 함.
3. tuple이야 쉽게 destructure가 되지만, `let (Ok((y, z)) | Err((y, z))) = x;`같은 거는 어떻게 풀 건데?
  - 이거 하는 김에 `Field::Name`도 좀 없애버릴까? 이거 때매 생기는 unwrap이 장난 아니게 많음...

destructure를 *안* 하면 문제가

1. hir이 돌기 위해서는 선언된 이름의 목록이 필요함!!
2. patterned-let을 저장하기 위해서 field를 추가하면... 뒤로 줄줄이 복잡해짐 ㅠㅠ

---

타협안

`let (x, y, _) = foo();`를 `let (x, y) = match foo() { (x, y, _) => (x, y) };`로 바꾸고

`let (Ok(y) | Err(y + 1)) = foo();`를 `let y = match foo() { Ok(y) | Err(y + 1) => y };`로 바꾸고

`let Person { name: _, age: x } = foo();`를 `let x = match foo() { Person { name: _, age: x } => x };`로 바꾸는 거임...

1. 일단 pattern에 bind된 name의 목록을 쭉 가져온 다음에 그 이름들만 가지고 tuple로 만드는 거지
2. `match`문 안에 pattern이 살아있기 때문에 mir에서 모든 검사를 다 할 수 있음.
3. 어쨌든 multi-name let을 만들어야 하니까 새로운 type이 필요하긴 함...
4. ...이렇게 할 바에는 그냥 `match`로 바꾸지 말고 `PatternLet { names: Vec<(InternedString, Span)>, value: Expr, pattern: Pattern }`하는게 낫지 않음??

---

이거랑 별개로, let destructure에도 type annotation 붙일 수 있게 하고 싶음... rust에서는 `let (x, y): (u32, u32) = foo();`처럼 함.

# 103. `ast::FullPattern::check()`

1. `CannotAnnotateType`
2. `NameCollision` -> same name is bound multiple times
3. `RedundantNameBinding` -> `a @ b`
4. `InclusiveRangeWithNoEnd`
5. `AstPatternTypeError`
  - `1..2..(3..4)`, `1..(2 | 3)`, `1..(2 ++ 3)`
    - 이건 어쨌든 ast에서 잡아야함. 별개의 ErrorKind를 만들까?
    - 생각해보니까 `1..(2 ++ 3)`은 나중에도 잡을 수 있는 거 아님??
    - 생각을 해보니까 `..`의 lhs/rhs로 올 수 있는 것들이 엄청나게 제한됨!!
      - wildcard는 못 오고 (와도 의미가 없으니 그냥 거절해버리자)
      - name binding도 못 오고
      - ... 걍 literal밖에 못 올 거 같은데? 그나마 dollar-ident 정도까지는 될 듯?
  - 잘 설계하면 전부 다 나중에 잡을 수 있는 거 아님??
  - `1..'a'`, `[] ++ 'a'`, `(1, 'a')`
    - 이런 건 전부 다 나중에 잡을 수 있음!!
6. `DifferentNameBindingsInOrPattern`

# 102. const expr in patterns

1. Parser는 arbitrary infix op를 처리 가능. error는 나중에 날릴 거임.
  - infix-op의 경우, `-`만 처리 -> 이거는 literal처럼 취급할 거임
  - `ast::Pattern::Concat`을 `ast::Pattern::InfixOp`로 바꾸면 됨.
2. 단, range는 여전히 지금처럼 처리. or도 여전히 지금처럼 처리 (이건 infix-op가 아님!).
3. 실제로 지원되는 infix op는 제약이 심함
  - lhs와 rhs가 모두 const인 경우 -> hir이나 mir에서 eval 해버리고 진행할 거임!
    - name binding이 붙어있으면 어떻게 뺄 거임??
  - lhs와 rhs 중 하나가 (ident | dollar ident)이고 나머지 하나가 literal인 경우
  - op가 concat이고 lhs와 rhs가 모두 list인 경우
  - op가 concat이고 lhs와 rhs 중 하나가 list이고 나머지 하나가 (ident | dollar ident)인 경우

# 101. code generator for error variants

1. Let's use a procedural macro.
2. A table with _ columns: variant name, level (error/warn), index
3. It generates the enum definition, endec, and `ErrorLevel::from_error_kind`.

# 100. `set!` and `map!`

In order to use Sodigy as a config language, we need map and set!

Let's use Rusty syntax: `map!( k1: v1, k2: v2 )` and `set!(v1, v2, v3)`...

Maybe we can do a pattern matching with these?

# 99. panicking is impure!!

```
fn check_all(xs) = match xs {
  [] => True,
  [x] ++ xs => {
    // `check` might panic
    let _ = check(x);
    check_all(xs)
  },
};
```

for 문이 없으니까 `check`를 저런 식으로 호출하고 싶은 유혹에 빠질 수 있음!! 근데 dead-code elimination을 하면 `check`를 호출을 안하고 넘어가게 되잖아? 그럼 안되지...

저렇게 하려면 `let _ = check(x);`대신 `assert check(x);`를 써야함!!

근데 이거를 사용자한테 알려주는게 무지하게 빡셈.

1. unused value에다가 무조건 이런 warning을 띄우면 오히려 더 헷갈림.
2. panickable function...을 추적하는 건 가능하지만 사실상 모든 함수가 panickable할 거여서 별 의미는 없음.

# 98. more on debugging

코드를 짜다 보니...

1. assert_panic이 필요함!! 옛날에 이런 이슈 있었던 거 같은데 ㅋㅋㅋ
2. debug 함수가 더 많이 필요 -> 이것도 분명히 옛날에 이슈 있었는데??
  - 일단, 아무 위치에서나 print 찍을 수 있게 만들어야 함!!
  - 옛날에 얘기 나왔던게, `echo` statement (not expression)를 만들까...였는데, 저거 만들면 분명 사람들이 `print`처럼 쓸 거여서 보류했음.
    - 아니면 이름은 `debug`라고 짓는 거임 ㅋㅋㅋ
  - 일단, 함수 진입할 때 log 찍는 decorator는 추가해야함!!
    - 근데 이것도 똑같은 문제 있는 거 아님?? 이것도 print처럼 쓸텐데 그럼 `echo`랑 뭐가 달라? 오히려 더 불편한 거 아님?? ㅋㅋㅋ ㅠㅠ
  - 그럼, 사람들이 `print`처럼 쓰면 문제가 뭐임??

# 97. more on bytecodes

1. `Const`를 어떤 식으로 저장?
  - Integer
    - `Const::Integer`를 할지 `Const::Compound`를 할지를 결정해야함
    - 동일한 integer literal이 여러번 나올 확률이 높음. 그럼 interning을 해야함?
  - Number
    - 일반적인 sodigy-struct랑 동일하게 처리할지, Number를 위한 variant를 추가할지를 결정해야함
    - number 구현을 바꾸는 compiler flag도 추가하고 싶음. 기본적으로는 ratio를 쓰지만 fixed point나 (software-implemented) floating point를 쓸 수도 있게...
  - Byte/Char
    - `Const::Byte`를 할지 `Const::Scalar`를 할지를 결정해야함
  - String
    - 일반적인 list랑 동일하게 처리할지, string을 위한 variant를 추가할지를 결정해야함
    - string을 위한 특수 처리를 한다면, utf-32를 쓸지 utf-8을 쓸지도 결정해야함
      - utf-8을 쓰면 runtime performance가 떨어지는 대신에 executable 크기가 작아짐
      - 둘다 가능하게 만들고 compile option에 따라서 바꿔쓸까?
    - 동일한 string literal이 여러번 나올 확률이 높음. 그럼 interning을 해야함?
  - Span
    - 일반적인 sodigy-struct랑 동일한 구조를 가졌으면 좋겠음... 아직 sodigy-struct의 구조가 확정이 안돼서 문제
  - Boolean
    - `Bool.True`는 항상 scalar 1이고, `Bool.False`는 항상 scalar 0이었으면 좋겠음...
2. executable에 들어가야 하는 정보에는 뭐뭐가 있지?
  - 각 func/assert/(top-level-)let의 bytecode
  - 모든 label을 static하게 만들기 vs `Label::Func(Span)`으로 남겨두기
    - 전자가 성능은 더 좋음. 차이 많이 날 듯?
    - 후자가 더 코드가 깔끔함. 후자가 library化가 쉬움.
  - entry point
    - top-level assertion의 위치들, main 함수

# 96. Defspan dependency graph in MIR level

We draw the dependency graph between def_spans, in MIR level. By doing this we can

1. warn unused names that are not checked in hir
2. maybe useful for some optimizations?
  - e.g. some context for inlining

# 95. dumping warning/errors

I implemented a deduplication for warnings/errors, but I just realized that it's useless.

Each process will dump errors and quit, so if different processes have the same error, we'll see duplicated errors!

Also, if there're 2 processes and one has warnings and the other has warnings and errors, we'll not see the warnings from the un-error process.

What I propose is:

1. Each session sends errors and warnings to the main process
2. the main process may dump the errors immediately, or defer
  - the main process deduplicates the errors
3. each session still remembers the errors and warnings after sending them to the main process, and they're encoded to the ir-cache
  - so that, it can retrieve warnings from cached sessions
  - we have to rely on the main process' deduplication!

# 94. trait system

How about an ad-hoc trait system?

A trait defines methods (without body) and fields (only types).

You don't explicitly implement a trait. If a type implements all the methods and fields, the type implicitly implements the trait.

It can help generate more readable error messages.

For example, when you define a generic. You might want to add a constraint: `T has to implement FromStr`. Then, the compiler will check if `T` meets the condition before monomorphizing the generic. If not, it'll throw an error.

# 93. update_compound, read_compound

1. 이름 변경: update_compound -> store, read_compound -> load
2. intrinsic으로도 두고 (sodigy가 직접 쓸 때도 있음), Bytecode로도 만들자 (최적화 용이)
  - `Bytecode::Store { ptr: Memory, offset: MemoryOrStatic, value: Memory }`
  - `Bytecode::Load { ptr: Memory, offset: MemoryOrStatic, dst: Memory }`

# 92. whether to drop

1. 함수에서 나가거나 block에서 나갈 때 일괄적으로 drop을 해야함
  - 함수 param type과 block let type은 접근이 쉽기 때문에 결정하기 쉬움
2. intrinsic을 호출한 다음에 arg를 즉시 drop해야함
  - 몇몇 intrinsic (AddInt, LtInt, ...)은 type을 알기 때문에 drop 할지말지 결정이 쉬움!
  - `fn init_list<T>(/* varargs */) -> [T]` -> 얘는??
    - 얘는 애초에 stack 쓰지말고 heap에다가 바로 올리고 싶은데?? 지금 bytecode로는 불가능!!
    - `Memory::Return`에다가 element 올리고, update_compound로 올리고... 이러면 불필요한 inc_rc, dec_rc가 들어감 ㅠㅠ
3. drop도 여러 종류가 있음 (아래 enum 참고)

```rs
enum DropType {
    // Byte, Char
    // No need for drop
    Scalar,

    // Int, (Byte, Byte)
    // Just decrement its rc.
    SimpleCompound,

    // List is very special because it
    //   1. has an arbitrary number of arguments
    //   2. has an integer for length
    // So, it has to drop the integer (which is SimpleCompound),
    // and the elements with the given DropType.
    List(Box<DropType>),

    // (Byte, [Char]), (Int, Int)
    Compound(Vec<DropType>),
}
```

# 91. `todo!()`, `panic!()`, `unreachable!()`

Rust에서 쟤네를 많이 쓰기 때문에 Sodigy에도 넣고 싶음!!

1. rust에서는 macro로 쓰는데 Sodigy에서는 굳이...
  - function으로 써도 되고 value로 써도 됨 (어차피 `!` value이니까)
2. Rust에서는 string formatting을 어떻게 하냐의 차이가 있지만, sodigy에서는 그런 차이 안 둘 거임!!
3. string을 optional하게 받기는 할 거임
4. panic하면 span도 보여줄 거야?? 어떻게 보여줌??
5. `todo()`가 있으면 compiler warning을 날릴까?? 괜찮은 듯?

# 90. enum representation

1. 첫번째 field에 variant가 들어가고, 나머지에 실제 값이 들어감
2. field를 하나만 쓸 거면 굳이 heap에 올릴 필요가 없음, 그냥 scalar값 하나 (32bit)만 써도 됨!
3. 더 최적화 하자면, `Option<Bool>`같은 애들도 variant 3개짜리 enum으로 취급해버리면 전부 scalar로 표현 가능!!

# 89. More on explainability

1. Type infer 할 때 기록 남기는 옵션 추가
  - type var에 정보를 추가할 때마다 그게 기록으로 남음
  - `solve_subtype()`이랑 `substitute()`에서만 호출해도 충분함!
2. dispatch_map이 생성될 때마다 기록 남기는 옵션 추가
3. 설명을 듣고 싶은 span을 고르면 전체 기록에서 그 span과 관련있는 기록만 뽑아냄
  - 이걸 시간 순으로 쭉 보면 뭐가 어떻게 되는지 쉽게 알 수 있을 듯?
4. 아니면, type error가 발생하면 기록 남기는 옵션을 켠 다음에 처음부터 mir-type을 다시 돌리는 거임!
  - 그럼 아주아주 자세한 에러 메시지를 남길 수 있음...

# 88. More on scalability

Inheritance도 아니고 composition도 아닌 새로운 방식을 택할 거임.

1. generic function은 C++처럼 ad-hoc으로 monomorphize하는 방식임. 그대신 컴파일 에러를 예쁘게 내려고 노력할 거임!!
2. poly generic은 지금 그대로
3. 어떤 type에 method를 추가하는 거는, 그 type이 정의된 project 안에서만 가능함!
4. extension이라는 개념이 있음. 어떤 type에 extension을 붙이면 그 type에 method를 추가할 수 있음.
  - 다른 project에서 정의된 type에도 extension을 붙일 수 있음.
  - extension으로 추가된 method를 사용하려면 그 extension을 import 해야함.
  - 서로 다른 extension이 동일한 method를 추가하면, 그 extension들을 동시에 import 하면 오류남!

# 86. more general generic system

```
#[poly]
fn print<T>(v: T) -> String = v.to_string();

#[impl(print)]
fn print_int(n: Int) -> String = f"int: {n}";
```

When you call `print("100")`, it will use the default implementation in the body of `print`. If you call `print(3)`, it'll call `print_int`.

You can also call `print_int` like normal functions.

```
#[poly]
fn to_string<T>(v: T) -> String;

#[impl(to_string)]
fn to_string_int(n: Int) -> String = match n {
    0 => "0",
    1 => "1",
    2 => "2",
    _ => panic(),
};
```

In this case, `to_string(0.5)` is a compile error because there's no implementation of `to_string` for `Fn(Number) -> String`.

```
// std
#[poly]
fn add<T, U, V>(a: T, b: U) -> V;

#[built_in]
#[impl(add)]
fn add_int(a: Int, b: Int) -> Int;

#[built_in]
#[impl(add)]
fn add_number(a: Number, b: Number) -> Number;

// user
3 + 4
```

It'll first convert `3 + 4` to `add(3, 4)`. The remaining is the same as user-defined polys.

---

1. Let's say there are 2 implementations for `add`: `Fn(Int, Int) -> Int` and `Fn(Number, Number) -> Number`.
  - When `Fn(Int, Int) -> Int` is given, ... easy!
  - When `Fn(Int, TypeVar(x)) -> Int` is given, it founds out that there's only 1 possible candidate. It also adds another type expression `TypeVar(x) = Int`.
  - When `Fn(Int, Number) -> TypeVar(x)` is given, it founds out that there's no possible candidate. We have to be very careful when generating error messages
2. Let's say there are 2 implementations for `index`: `Fn([T], Int) -> T` and `Fn(Map<K, V>, K) -> V` (TODO: generics in `Fn` types)
  - We have to build a statemachine for this...
3. Let's say there are a lot of implementations for `map` and 2 of them are `Fn(Option<T>, Fn(T) -> U) -> U` and `Fn(Option<Int>, Fn(Int) -> T) -> T`.
  - When `Fn(Option<Int>, Fn(Int) -> TypeVar(x)) -> TypeVar(x)` is given, we have _ choices:
    - silently choose more concrete one
    - asks the user to specify one
    - throw a compile error (there are multiple candidates)

# 84. methods and traits

1. syntax
  - `impl<T> Option<T> { .. }`, `impl Option<Int> { .. }`
  - `#[impl(Option<T>)] fn map<T>(opt: Option<T>, f: Fn(T) -> U) -> Option<U>;`
2. adding a method to a foreign type
3. traits
  - we need composition or inheritance if we want BIG sodigy projects
4. If there're same methods for `Option<T>` and `Option<Int>`, I want the compiler to choose more concrete one.
5. How about generic-based type classes?
  - Generic functions are compile-time-type-checked-duck-typing. Let's say `map(s)` expects `s` to implement some methods and some fields. Then the programmer calls `map(3)`. If `Int` satisfies all the requirements, there'd be no compile error. Otherwise, the compiler will give a very nice error message.
  - This is nice, but, there's a problem. I want the functions to be chained with dots, like `a.b().c().d()`, but with this approach, I'll fall into a parenthesis hell...
  - How about a syntax that turns an arbitrary function into a method (connecting a function with a type)
6. struct-constants, like `impl f32 { const PI: f32 = 3.1415; }` in Rust.
  - how about struct-structs, struct-enums, etc?

아니면 이건 ㅇㄸ `a.b(c).d(e)`는 `d(b(a, c), e)`의 syntax sugar임 -> 무조건 풀고 시작하는 거!!
-> 사실 이거는 단점이 너무 뚜렷함. syntax sugar로 써버리면 똑같은 namespace를 공유해야하잖아 ㅠㅠ
-> 86번에서 논의 중인 trait system을 잘 활용하면 namespace 문제가 없을 수도 있음!!

# 83. unused warnings

1. 한 함수에서 param 3개 정의하고 셋다 안 쓰면? 경고를 한번에 날리는게 낫지 않나?
  - unused params: `a`, `b` and `c`
  - span도 한번에 보여주는게 더 이쁨!
  - 근데... 한 함수인지 아닌지를 어떻게 판별해? 함수가 아니라 use같은 경우에도 `use std.prelude.{A, B, C};`에서 경고 뜨면 합치고 싶음!!
  - 단순히 span이 가까운지로 확인하기?? vs 한 group에 속하는지를 꼼꼼히 검사하기
    - 한 group에 속하고 span도 가까운 경우: 합치는게 맞음!
    - 한 group에 속하는데 span이 먼 경우: 합쳐도 그만 안 합쳐도 그만
      - 근데, 한 group에 경고가 여러개 뜨면 걔네가 전부 하나로 합쳐지거나 전부 갈라지거나 해야 예쁘지 애매하게 합치면 이상할 거 같은데?
    - 다른 group에 속하는데 span이 가까운 경우: 합치면 무지 이상함
    - 가까운지 아닌지 확인하는 것도 매우 애매: 함수 param에서는 type annotation이랑 default value때문에 거리가 꽤 멂...
      - 더 깊게 들어가자면, 나처럼 param 목록에서 newline을 남발하는 사람들은 span이 아무리 멀어도 하나로 합치면 이뻐짐 ㅋㅋㅋ
  - 잠깐 관찰해보니 rust는 함수 param은 안 합치고 use는 합치네.
2. top-level에서 정의된 item인 경우 unused인지 아닌지 알기 빡셈
  - 완전 private한 경우, 지금의 logic으로 다 잡을 수 있음!!
    - 아닌가, 생각해보니까 private이어도 하위 모듈에서는 쓸 수 있잖아...ㅜㅜ
  - public한 경우, 다른 module에서 어떻게 쓰는지 다 뒤져봐야함.
    - 지금은 이거 검사할 수 있는 장소가 아예 없음...!!
    - 그나마 inter-hir?? inter-hir에 visibility 검사 자세하게 하도록 수정하면 검사할 수 있을 듯!
  - 어차피 inter-module로 검사할 거면 intra-module에서 검사할 필요가 없는 거 아님..?? ㅋㅋㅋ

# 82. inter-mir

1. in order to type-check,
  - it needs `types: HashMap<Span, Type>`, `generic_instances: HashMap<(Span, Span), Type>`, `solver`, `lang_items: HashMap<String, Span>` and items (`&[Func]`, `&[Let]`, ...).
    - `types` and `solver` must be separated in order to avoid mut-ref issues.
    - `generic_instances` and `solver` must be separated for the same reason.
    - currently, `solver` has `lang_items` field. It doesn't matter who has this field.
    - currently, `solver` collects the errors and warnings, and passes it to mir-session in the end.
    - mir-session or `solver` might have to create `span_string_map` for error messages
      - the map is global, and has to be generated once (or never).
    - `types` and `generic_instances` have to be global, while items can be local.
    - we can't run it in parallel, because the global `types` and `generic_instances` have to be updated.
2. We might do extra checks or analysis. We have to implement that in inter-mir pass.
3. All the optimizations must come after type-check, hence in inter-mir.

---

그럼 inter_mir_session이랑 mir_session이랑 type_solver를 다 따로 해야할 거 같은데?? ㅜㅜㅜ

생각해보니까 items가 `&[Item]`이 아니고 `&mut Vec<Item>`임!! monomorphize를 하거나 optimization을 하면 수정해야하잖아...
그럼 좀 나음. mir_session을 하나로 합쳐버린 다음에 작업하면 됨!!

# 81. new issues in inter-hir

```
use std.{Bool, Int};

fn add(a: Int, b: Int) -> Bool = a + b;
```

The return type of `add` is `Bool`, but it returns an `Int`. The error message should underline the type annotation and the expression. But the problem is that the span of the type annotation is replaced with `Bool` in `use std.Bool;`...

We should only replace the def_span but we're replacing the def_span and span.

More on this

```
use x.z as a;
use y as x;
```

Let's say `y` doesn't have an item named `z`. Then, it has to underline `x.z` and say "module `x` doesn't have an item named `z`."

-> just replace the def_span of `x` in `x.z`

How about...

```
use x.y.z as w;
use a.b.c as x;
```

`use x.y.z as w;` would become `use a.b.c.y.z as w;`. What if `a` doesn't have an item `b`? It'd generate the same error twice. We have to prevent that.

It's impossible to underline `a.b.c.y.z` because if `a` is using `x`'s span, then `a.b` doesn't make sense. If all spans are conserved, `c.y` doesn't make sense.

How about, `a`, `b` and `c` all use `x`'s span?

---

정리...

1. alias를 풀 때는 def_span만 갈아끼우고 span/id는 그대로 둔다.
2. field가 있는 경우 `x.y.z`를 `a.b.c.y.z`로 갈아끼웠을 때는 `b`와 `c` 모두 `x`의 span을 물려받는다.
3. `use a.b.c as x;`에서 `a.b`가 오류일 경우 `x`를 참조하는 모든 곳에서 오류가 나겠지? 다른 곳에서는 오류가 나지 않도록 미리 방지해야함..!!

# 80. Language doc

1. I'm writting the document at `spec.md`. I'll have to split files before it gets too long.2
2. I want to implement a markdown parser in Sodigy to parse the document.
3. I want to run the codes in the document's code blocks.
  - Some blocks assert that they don't compile. Some assert that they compile but don't pass the test.
  - I want it to create a new code block with the compile error messages (colored).

# 79. Commit hash

사실 Sodigy랑은 큰 상관없고 그냥 심심풀이용임.

1. `sodigy version`을 하면 commit hash가 나오게 하고 싶음!
2. 보통은 `build.rs`를 이용해서 commit hash를 집어넣음
3. 왜냐면 commit hash를 hard-code하는 순간 commit hash가 바뀌어버리기 때문에 hard-code할 수가 없거든
  - ... 그렇지 않음!! 비트코인 채굴하는 거랑 똑같은 원리로 넣을 수 있음. commit hash를 무작위로 hard-code 하다보면 언젠간 일치하거든!!
  - (commit hash 변경, `git add <file>`, `git commit --amend`, commit hash 확인) -> 이거를 계속 loop 돌리면 됨!!

```py
# params
file = "src/lib.rs"
line = "pub const COMMIT_HASH: &'static str = \"{{replace}}\";"

import subprocess
rep_at = line.index("{{replace}}")
prefix = line[:rep_at]
suffix = line[(rep_at + len("{{replace}}")):]

with open(file, "r") as f:
    lines = f.read().split("\n")

line_no = [i for i, line in enumerate(lines) if line.startswith(prefix) and line.endswith(suffix)][0]

for i in range(4096):
    hash = f"{i:03x}"
    new_line = line.replace("{{replace}}", hash)
    lines[line_no] = new_line

    with open(file, "w") as f:
        f.write("\n".join(lines))

    subprocess.run(["git", "add", file], check=True)
    subprocess.run(["git", "commit", "--amend", "--no-edit"], check=True)
    real_hash = subprocess.run(["git", "rev-parse", "HEAD"], check=True, capture_output=True, text=True).stdout

    if real_hash.startswith(hash):
        break
```

이렇게 하니까 너무 오래 걸림... 4096개 도는데 몇분은 걸리는듯 ㅠㅠ
또다른 문제: 4096개 다 돌았는데 collision이 하나도 없을 수도 있음!
또다른 문제: `.git/`에 쓰레기가 조금씩 쌓임 -> 이건 사소

Rust로 짜면 더 빨리 짤 수 있을 거 같기도 하고??

1. `git cat-file commit <hash>` 하면 현재 commit의 정보가 나옴. 아마 이거 hash하면 그대로 commit hash 될텐데?
  - ㄴㄴ perplexity한테 물어보니까 `"commit " + content.len() + "\0" + content` 한 다음에 hash해야한대. 참고로 content.len()은 byte로 계산
2. tree도 마찬가지래 `"tree " + content.len() + "\0" + content` 해야한대...

# 78. Generic functions with default values

`fn add<T, U, V>(a: T = 1, b: U = 2) -> V = a + b;`

... 이러면 `T`는 항상 Int라고 봐야돼?? 그건 아니긴한데 좀 이상하네

그냥 금지해버릴까??

# 77. Sodigy for real-world programming

In order for Sodigy to be practical, it needs impure functions.

1. Simple File IO
  - read/write/append to file (string/bytes), read dir, create dir, remove dir, exists, create file, remove file
  - We don't have this, but we definitely need this.
2. Time
  - sleep, get time
  - We don't have this, but we definitely need this.
3. Random
  - get random value
  - We don't have this, but we definitely need this.
4. Fancy File IO
  - copy_file, rename, set_current_dir, get_current_dir, file_metadata
  - Maybe later...
5. Network
  - http request/response
  - it'd be nice, but it'd be much harder to add new backends
6. GUI
  - input events (keyboard, mouse, window), output events (draw something)
  - it'd be a lot of work...
7. DOM manipulation
  - purescript is Haskell-ish javascript, and Sodigy becomes Rust-ish javascript!
  - it'd be a lot of work...
8. SQL
  - We have 2 choices: C FFI or implement new DBMS from scratch

There are some pure functions (libraries) that Sodigy is missing

1. Regex
2. JSON/binary serde
3. Markdown

It'd be nice to have multithread/multiprocess capabilities, but it's not just about libraries, we have to tweak the runtime...

1. `spawn(\(x) => foo(x))` to spawn a new thread/process.
  - It's easy, but how do they interact with each other?
2. async/await -> we need a built-in event loop...

# 76. Subtyping...

1. Never type만 고려할 경우
  - `Never` is a subtype of everything
  - `Never`를 위한 variant와 (`Type::Never`) notation (`!`)을 새로 만들어야 함
  - assertion이나 if처럼 특정 type을 기대하는 경우: 해당 type의 subtype이 나오면 맞다고 하고 넘어가기
    - 함수 param도 이에 해당
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
  - 또 어디에 필요하려나...
  - 하고싶기는 함. `enum Foo`가 `variant A, variant B, variant C`를 갖는데 `Foo`를 return하는 어떤 함수가 항상 `A` 혹은 `B`를 return 하는 경우: `C`가 나올 수 없다는 걸 type checker가 잡고 싶음.

# 72. Visibility

가라로 하던 거 업보 청산할 시간...

1. 지금은 inter-hir에서 `iter_public_names`를 한 다음에, public한 name들만 item_name_map에 올려둠.
  - 코드가 돌아야하니까 지금은 일단 모든 item을 public하다고 가정하고 풀어버리는 중!
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
2. function parameter
  - `_`로 시작하는 이름은 unused_name 안 날리기? -> 이거 구현하면 사실 그냥 identifer랑 다를게 없음
  - 이것도 살짝 더 생각해야함. `_`로 된 func param 여러개 선언하면 오류 날릴 거임?
    - 와 rust에서는 `_`로 된 func param 여러개 선언하는 거 가능하네!!
  - 그럼 `foo(3, _=4, _=5)` 하면 오류 날려야하는데??
3. type annotation
  - 여기서는 좀 special treatment가 필요함! 어차피 special treatment 할 거면 아예 구분하자 이거지

생각해보니까 identifier가 쓰이는 모든 곳을 다 고쳐야함... 흠 좀 빡셀 거 같기는 한데 ㅠㅠ

그럼 `_`로 시작하는 이름은 unused_name 안 날리는 것만 구현하자!

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

Perplexity한테 왜 `TryFrom<Foo> for String`/`TryFrom<String> for Foo`을 안 쓰고 `ToString`/`FromStr`를 쓰는지 물어보니까 `ToString`하고 `FromStr`이 더 먼저 존재했기 때문에 backward compatibility 때문에 건드릴 수가 없었대...

---

new draft

`x as T`, `x as? T`로 type conversion (not casting, which is reinterpretation of the same bit pattern and not coercion, which is implicit) 구현하자!! 둘다 poly로 구현하면 됨: `#[poly] fn convert<T, U>(v: T) -> U; #[poly] fn try_convert<T, U, E>(v: T) -> Result<U, E>;`

1. 현재 문법으로는 poly 표현이 살짝 빡셈: `x as Int`를 `convert(x)`로 바꾸면 `Int`라는 정보가 사라짐... 결국에는 `convert.<Int>()`로 해야하는데... turbo-fish 문법이 아직 미완성 ㅠㅠ
2. `x as _`로 해도 됨?
  - 이러면 implicit type conversion 아님?
  - 생각해보니까 rust에서도 그냥 `x.into()`로만 쓰는 경우 많잖아...
3. `as`랑 물음표 사이에 띄어쓰기 있으면 한 토큰으로 잡아?
4. 이거 하면 `<`랑 `<<` 더 잘 구분해야함 ㅠㅠ Rust에서 왜 그런 에러메시지 날리는지 알겠네...
5. 이거 하는 김에 poly에서 에러메시지 훨씬 더 섬세하게 날리게 해야함!! infix op 없을 때나 conversion 안 될 때 dedicated error message 만들어!!

# 61. more on purity

How do you define purity?

1. if x = y, then f(x) = f(y)
  - 참고로 user-defined `=` operator랑은 상관없음!! 그냥 overloading 할 수 있게 열어주자.
2. no side effects
  - How do you define side effect?
3. is `panic()` pure?

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
  - 그러려면 operator도 일반 generic function처럼 처리해야함. 그러려면 operator의 generic parameter의 def_span을 나타낼 방법이 있어야 함!!
  - 이렇게 하면 코드가 훨씬 간단해짐 `infix_op_type_signatures` 이딴 거 없어도 되거든 ㅋㅋㅋ
  - 생각해보니까 이거 하면 `Callable::GenericInfixOp`도 사라짐!!
    - 오
- 근데 어차피 monomorphize를 할 거면, monomorphize 한 다음에 그 안에서 새로 type-check하면 안됨 (C++ 방식)? 이게 덜 복잡할 거 같은데... 이걸 하려다가 포기했던 이유가
  - 1, error message가 난해해짐.
  - 2, generic function body 안에 type variable X가 있다고 하자, 이 function이 instantiate 될 때마다 X가 하나씩 늘어나야함. X들끼리 서로 다르게 type-infer 해야하거든... 그럼 코드가 엄청 복잡해짐.
    - 간단할 거 같은데? generic function을 한번에 하나씩만 type-check를 하고, 각 function의 type-check가 끝날 때마다 그 안에 있는 type variable과 관련된 정보는 다 삭제하면 됨!!
    - body 안에 있는 type variable의 목록을 알아내는게 중요하겠네!
      - 단순 삭제만 하면 안되고, infer에 실패한 type variable이 있는지도 검사해야함
  - ㄴㄴ 걍 아예 새 function을 만들어버리고 span도 다 새로 주자. 이게 근본적인 해결책 아님?
- Rust 방식은 하고싶지 않음. 그렇게 하려면 trait system을 완전 정교하게 design 해야하거든...

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
    - 그럼 이름을 `debug`로 바꾸면 되지 ㅋㅋㅋ
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
2. params and flags
  - a command takes a small number of params (can be zero) and a lot of flags
  - you can use parenthesis to make params less ambiguous
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
  - the current runtime has no type information... so if we pass an integer to a function that expects a string, it'll behave in a really weird way but doesn't throw any error
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

근데 이거를 하려면 for문을 만들어야하는데...

---

또다시 정리 ㅋㅋㅋ 쟁점들

1. 기존 Sodigy compiler를 얼마나 재활용할 것인가?
2. 예외처리를 어떻게 할 것인가
3. inline expression을 허용할 것인가
4. for문을 구현해야하나
5. type check를 언제 할 것인가
  - type check를 할지말지는 선택사항이 아님... runtime에라도 해야지...
  - 만약에 runtime에 할 거면 Sodigy가 내놓은 값을 enum으로 감싸서 (`serde_json::Value`처럼) 써야함
  - compile time에 할 거면 기존 type checker를 재활용해?
    - 재활용하기에는 기존 type checker가 너무 무겁고 (inference는 필요가 없거든), 새로 만들기에는 너무 중복되는게 많음
    - 재활용하려면 hir->inter-hir->mir을 전부 다 태워야 함

---

아니면 이건 ㅇㄸ

`main.sdgcmd`가 따로 있음. Sodigy와 완전히 동일한 문법을 사용하지만 몇몇 impure function을 추가로 사용할 수 있고, `#[impure]`를 이용해서 impure function을 정의할 수 있음.

module hierarchy를 잘 만들면 impure context를 완전히 격리시키는게 가능 (일반 sodigy 파일에서는 impure function을 사용 불가)

Action을 순서대로 실행하기 위해서는 `exec_actions(a1, a2, ...)`가 필요!! 모든 action을 주어진 순서로 실행한 다음에 제일 첫번째 값을 반환 (제일 마지막 값을 반환하는 함수도 만들어야할 듯?). -> 이거 std에서도 써먹을 수 있을 거 같은데??

아마 최적화를 구현하면 pure-function을 상정한 최적화가 많이 들어갈텐데, 걔네를 잘 걷어내는게 관건!

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
  - 생각보다 안 느릴 거같은데?? 그냥 call stack 깊이만 보고 하면 안됨??

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
2. Inline assertions
  - It's like `assert!` in Rust.
  - In release mode, inline assertions are disabled.
3. Name-analysis: We have to tweak some logic.
  - If a name is only used by assertions, but not by expressions, we raise an unused name warning.
    - But we add an extra help message here, saying that the name is only used in debug mode.
    - How about adding `#[unused]` decorator?
  - If a name is used by expressions only once, and multiple time by assertions, we inline the name anyway. For example, `{ let x = foo() + 1; assert x > 0; assert x > 1; [x] }` becomes `{ let x = foo() + 1; assert x > 0; assert x > 1; [foo() + 1] }`.
    - We need a lot of tweaks here...
    - `let x` statement is removed in release mode, but not in debug mode.
4. Assertions that are enabled in release mode.
  - How about `#[always]` decorator?
  - If a top-level assertion is decorated with `#[always]`, it's asserted before entering the main function.
    - It's treated like a normal test in test context.
5. Syntactic sugar for `assert x == y;`
  - 이게 실패하면 lhs와 rhs를 확인해야함...
  - 근데 syntax 기준으로 뜯어내는 거는 너무 더러운데... ㅜㅜ 이건 마치 `==`를 syntactic sugar로 쓰겠다는 발상이잖아 ㅋㅋㅋ
  - 아니면 좀 덜 sugarly하게 할까? 그냥 모든 expr에 대해서 다 inspect 하는 거임 ㅋㅋㅋ
    - value가 `Call { func: Callable, args: Vec<Expr> }`인 경우, `func`랑 `args`를 dump (infix_op도 다 여기에 잡힘)
    - value가 `Block { lets: Vec<Let>, value: Expr }`인 경우, `lets`를 dump (expr만), `value`는 dump할 필요없음 (당연히 False일테니)
    - value가 `if { cond: Expr, .. }`인 경우, `cond`를 dump, `value`는 dump할 필요없음 (당연히 False일테니)
    - value가 `match { value: Expr, .. }`인 경우, `value`를 dump하고 어느 branch에 걸렸는지도 dump
6. pre/post assertions
  - 함수 진입할 때마다 특정 assertion을 자동으로 호출하거나 함수 나갈 때마다 특정 assertion을 자동으로 호출하는 기능
    - 생각해보니까, 함수 나갈 때마다 assertion 호출하면 tail-call이 안되는데??

# 25. Make it more Rust-like!! ... 하다가 생긴 문제점

Name binding에 `$`를 안 붙이니까 한가지 문제가 생김: `True`랑 `False`에 match 하려면 `$True`, `$False`를 해야함... Rust는 `true`/`false`가 keyword여서 이런 문제가 없음.

-> 생각해보니까 이것도 안되네. `$True`면은 "True라는 이름을 가진 변수와 값이 같다"라는 뜻이잖아...
  - 아니지, 이미 namespace에 `use Bool.True as True`가 있으니까 `$True`로 해도 되지!
-> 할 거면 `Bool.True`로 해야함.

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
  - `Fn(Int, Int) -> Int` 자리에 `fn foo(x: Int, y: Int, z: Int = 5) -> Int`를 넣는 경우, `\(x: Int, y: Int) -> Int => foo(x, y, 5)`로 자동으로 바꾸기...??

1, 2, 3은 구현했고 4는 아직 미정
