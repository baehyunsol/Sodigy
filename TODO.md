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
