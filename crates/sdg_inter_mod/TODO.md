inter-module name resolving

```rust
enum ModuleOrDef {
    Def(UID),
    Module(HashMap<InternedString, ModuleOrDef>),
}

root: HashMap<InternedString, ModuleOrDef>;
everything: HashMap<UID, FuncDef>;
```

when compiler encounters a path `a.b.c`, it queries `root[a][b][c]` to get the def ID of `a.b.c`. then it replaces `ExprKind::Path(a, b, c)` to `ExprKind::Value(ValueKind::Def(uid))`.
