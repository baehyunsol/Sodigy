use super::super::AST;
use crate::session::LocalParseSession;

/*
```
{
    a = \{n, if n > 0 { a(n - 1) } else { 0 }};

    a
}
```

The name resolver thinks that `a` is a closure, because it's referencing `a`, which is not in the lambda's name scope.
But it's obvious that `a` is not a closure. This pass visits all the exprs, finds such cases, and fixes them.
It also deals with mutually recursive cases

1. if it finds `Call(@@LAMBDA_ABCDEF, a)`, which is a closure, it checks whether all of the arguments (captured vars) are functors
2. if so, it changes `Call(@@LAMBDA_ABCDEF, a)` to `@@LAMBDA_ABCDEF` and modify the def of `@@LAMBDA_ABCDEF` in AST

This pass must be called after name_resolve and before block_clean_up because,
1. name_resolve creates lambda definitions
2. block_clean_up will reject recursive lambda functions (unless this pass) because they reject recursive block defs
*/

impl AST {
    pub fn resolve_recursive_funcs_in_block(&mut self, session: &mut LocalParseSession) {
        todo!();
    }
}
