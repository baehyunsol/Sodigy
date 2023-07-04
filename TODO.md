```
def PI: Number = 3.1415;

def Add(a: Int, b: Int): Int = a + b;

def List(T: Type): Type = # What here...??

def NumList: Type = List(Number);

def add_first_two(ls: NumList): Number = ls[0] + ls[1];
```

`use A.B;` -> `use A.B as B;`
`use A.B.C;` -> `use A.B.C as C;`
`use A.B, C.D;` -> `use A.B; use C.D;`
`use A.{B, C, D};` -> `use A.B; use A.C; use A.D;`
`use A.{B as C, D as E}` -> `use A.B as C; use A.D as E;`

---

Parsers of block_expr and lambda_def rely on the fact that commas and semi-colons do not appear in expressions. They only appear inside `{}`s, `[]`s or `()`s. -> How do I guarantee that using code?