Again, ... ... ...

0. Everything is pure!!!
1. Pattern matching, like that of Rust
2. scoped block
  - `{ lex x = 3; let y = 4; x + y }`
  - can be nested -> each has its own name scope
  - A file is also a scope. Syntactically, there's no difference at all!!
  - let statement
    - `let x = 3;`
    - pattern desugaring `pat ($x, $y) = foo();`
    - function `func adder(x: Int, y: Int) -> Int = x + y;`
    - struct `struct Person = { age: Int, name: String };`
    - enum `enum Option(T) = { Some(T), None }`
3. structs and enums -> like those of Rust
4. doc comments and decorators -> like those of Sodigy
5. Everything is just a number
  - no integer, no float, no overflow, no underflow... it's just a number!
  - it uses rational numbers. it's total as long as you're only doing +-*/
  - e.g. a numeric literal `1.5 * 2.5` is parsed to `ratio(3, 2) * ratio(5, 2)` at compile time
    - more rooms for optimization!!
6. formatted strings... maybe?
7. incremental compilation
  - A compiler with a lot of passes
8. String is just String
  - The only thing that I don't like in Rust is `&str` vs `String`.
  - It's an array of characters, where a character is just an integer

---

좀 과하긴 하지만...

lexer까지는 rust랑 호환되게 만들어두고, 나중에 재활용할까? ㅋㅋㅋ
