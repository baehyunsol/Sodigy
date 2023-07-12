use super::super::AST;
use crate::expr::{Expr, ExprKind};
use crate::stmt::ArgDef;
use crate::value::ValueKind;

/*
```
{
    x = FOO;
    [x, x]
}
```

into

```
[FOO, FOO]
```

then inline `FOO`.

what if `FOO` is expensive?

making `FOO` lazy_static: what if it uses too much memory?
*/

/*
{
    a = foo();
    b = a;

    b + b
}

-> is `a` used only once? it seems so, but it's not!
*/

impl AST {

    // 1. If a definition is used only once, the value goes directly to the used place.
    // 2. If a definition is used 0 times, it's removed.
    // 3. If a value of a definition is simple, all the referents are replaced with the value.
    //   - simple value: single identifier (or a path), small number (how small?)
    // 4. If a block has no defs, it unwraps the block.
    // 5. Check cycles?
    pub fn clean_up_blocks(&mut self) {

        for func in self.defs.values_mut() {
            func.ret_val.clean_up_blocks();
            func.ret_type.clean_up_blocks();

            for ArgDef { ty, .. } in func.args.iter_mut() {
                ty.clean_up_blocks();
            }

        }

    }
}

impl Expr {

    pub fn clean_up_blocks(&mut self) {
        self.kind.clean_up_blocks();
    }

}

impl ExprKind {

    pub fn clean_up_blocks(&mut self) {
        match self {
            ExprKind::Value(v) => match v {
                ValueKind::Identifier(_)
                | ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_) => {},
                ValueKind::List(elements)
                | ValueKind::Tuple(elements) => {
                    for element in elements.iter_mut() {
                        element.clean_up_blocks();
                    }
                },
                ValueKind::Block { defs, value } => todo!(),
                ValueKind::Lambda(args, val) => todo!(),
            },
            ExprKind::Prefix(_, v) => v.clean_up_blocks(),
            ExprKind::Postfix(_, v) => v.clean_up_blocks(),
            ExprKind::Infix(_, v1, v2) => {
                v1.clean_up_blocks();
                v2.clean_up_blocks();
            },
            ExprKind::Branch(c, t, f) => {
                c.clean_up_blocks();
                t.clean_up_blocks();
                f.clean_up_blocks();
            },
            ExprKind::Call(f, args) => {
                f.clean_up_blocks();

                for arg in args.iter_mut() {
                    arg.clean_up_blocks();
                }

            }
        }
    }

}