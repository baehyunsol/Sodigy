# 3. DocComments and Decorators

1.

# 2. String literal

1. Char vs String
  - Integer vs List of Integer
2. Single Quote vs Double Quote
  - Char vs String
3. Formatted String (no Char)
4. Binary String (or Char)
5. Raw String (no Char)
  - 1. A string contains double-quote characters
  - 2. A string contains back-slash characters
6. Regex String
  - If I'm to pattern-match regex, I need a special syntax for regex literals.
7. Combination
  - e.g. format + binary

---

중간 정리

A string literal starts with N (odd number) double quotes and ends with the same number of double quotes. If it's prefixed with `r`, all the escapes are ignored.

In pattern matching, `r`-prefixed strings are treated specially.

1. normal string/char: rust-like
2. binary string/char: rust-like
3. format string: python-like
4. multi-quote string
  - if it starts with N double quotes, it has to end with N double quotes.
  - 
5. regex string
  - backslashes don't have any effect
    - if you want to use 
  - if it's in a pattern, it's a regex pattern
    - I want to bind names to its groups, but how?
    - How about `r @ r"(\d+)x(\d+)"` and use something like `r._0`, `r._1`.
      - `r` is a tuple: `(Option(String), Option(String), Option(String))`

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
