inter-module name resolving

```rust
enum ModuleOrDef {
    Def(UID),
    Module(HashMap<InternedString, ModuleOrDef>),
    Enum(HashMap<InternedString, UID>),  // it makes `use Bool.True as true;` possible
}

root: HashMap<InternedString, ModuleOrDef>;
everything: HashMap<UID, FuncDef>;
```

when compiler encounters a path `a.b.c`, it queries `root[a][b][c]` to get the def ID of `a.b.c`. then it replaces `ExprKind::Path(a, b, c)` to `ExprKind::Value(ValueKind::Def(uid))`.

when it sees `Option.Some`, it replaces it with the UID of `Option.Some`. when it sees `Option.Somme`, it replaces `Option` with the UID of `Option`. Though it's a name error, the error will be caught later. since `Option` may have a method or a field named `Somme`, we have to wait until it's method and field names are available.
