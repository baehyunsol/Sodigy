use crate::Session;
use sodigy_error::{Error, ErrorKind, NotExprBut};
use sodigy_hir::{CallArg, Expr, ExprOrString, Path, StructInitField};
use sodigy_name_analysis::{NameKind, NameOrigin};

impl Session {
    pub fn resolve_expr(&mut self, expr: &mut Expr) -> Result<(), ()> {
        match expr {
            Expr::Path(p) => {
                self.resolve_path(p, None, &mut vec![])?;

                if p.fields.is_empty() {
                    Ok(())
                }

                else {
                    *expr = Expr::Field {
                        lhs: Box::new(Expr::Path(Path {
                            id: p.id,
                            fields: vec![],
                            types: vec![None],
                        })),
                        fields: p.fields.to_vec(),
                        types: p.types.to_vec(),
                    };
                    Ok(())
                }
            },
            Expr::Number { .. } |
            Expr::String { .. } |
            Expr::Char { .. } |
            Expr::Byte { .. } => Ok(()),
            Expr::If(r#if) => match (
                self.resolve_expr(&mut r#if.cond),
                self.resolve_expr(&mut r#if.true_value),
                self.resolve_expr(&mut r#if.false_value),
            ) {
                (Ok(()), Ok(()), Ok(())) => {
                    if let Some(pattern) = &mut r#if.pattern {
                        self.resolve_pattern(pattern)
                    }

                    else {
                        Ok(())
                    }
                },
                _ => Err(()),
            },
            Expr::Match(r#match) => {
                let mut has_error = false;

                if let Err(()) = self.resolve_expr(&mut r#match.scrutinee) {
                    has_error = true;
                }

                for arm in r#match.arms.iter_mut() {
                    if let Err(()) = self.resolve_pattern(&mut arm.pattern) {
                        has_error = true;
                    }

                    if let Some(guard) = &mut arm.guard {
                        if let Err(()) = self.resolve_expr(guard) {
                            has_error = true;
                        }
                    }

                    if let Err(()) = self.resolve_expr(&mut arm.value) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::Block(block) => {
                let mut has_error = false;

                for r#let in block.lets.iter_mut() {
                    if let Err(()) = self.resolve_let(r#let) {
                        has_error = true;
                    }
                }

                for assert in block.asserts.iter_mut() {
                    if let Err(()) = self.resolve_assert(assert) {
                        has_error = true;
                    }
                }

                if let Err(()) = self.resolve_expr(&mut block.value) {
                    has_error = true;
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::Call { func, args, .. } => {
                let mut has_error = false;

                if let Err(()) = self.resolve_expr(func) {
                    has_error = true;
                }

                for arg in args.iter_mut() {
                    if let Err(()) = self.resolve_expr(&mut arg.arg) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::FormattedString { elements, .. } => {
                let mut has_error = false;

                for element in elements.iter_mut() {
                    if let ExprOrString::Expr(e) = element {
                        if let Err(()) = self.resolve_expr(e) {
                            has_error = true;
                        }
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::Tuple { elements, .. } |
            Expr::List { elements, .. } => {
                let mut has_error = false;

                for element in elements.iter_mut() {
                    if let Err(()) = self.resolve_expr(element) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::StructInit { constructor, fields, .. } => {
                let mut has_error = self.resolve_path(constructor, None, &mut vec![]).is_err();

                for field in fields.iter_mut() {
                    if let Err(()) = self.resolve_expr(&mut field.value) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::Field { lhs, .. } => self.resolve_expr(lhs),
            Expr::PrefixOp { rhs: hs, .. } |
            Expr::PostfixOp { lhs: hs, .. } => self.resolve_expr(hs),
            Expr::FieldUpdate { lhs, rhs, .. } |
            Expr::InfixOp { lhs, rhs, .. } => match (
                self.resolve_expr(lhs),
                self.resolve_expr(rhs),
            ) {
                (Ok(()), Ok(())) => Ok(()),
                _ => Err(()),
            },
        }
    }

    pub fn check_expr_path(&mut self, expr: &Expr) -> Result<(), ()> {
        fn check_path(path: &Path) -> Result<(), Error> {
            match path.id.origin {
                NameOrigin::FuncParam { .. } => Ok(()),
                NameOrigin::GenericParam { .. } => Err(not_expr_error(path, NotExprBut::GenericParam)),
                NameOrigin::Local { kind } |
                NameOrigin::Foreign { kind } => match kind {
                    NameKind::Let { .. } |
                    NameKind::Func |
                    NameKind::EnumVariant { .. } |
                    NameKind::FuncParam |
                    NameKind::PatternNameBind |
                    NameKind::Pipeline => Ok(()),
                    k => Err(not_expr_error(path, k.into())),
                },
                NameOrigin::External => unreachable!(),
            }
        }

        fn check_struct_path(path: &Path) -> Result<(), Error> {
            match path.id.origin {
                // what error?
                _ if !path.fields.is_empty() => todo!(),
                NameOrigin::FuncParam { .. } => Err(not_struct_error(path, NotStructBut::Expr)),
                NameOrigin::GenericParam { .. } => Err(not_struct_error(path, NotStructBut::GenericParam)),
                NameOrigin::Local { kind } |
                NameOrigin::Foreign { kind } => match kind {
                    // TODO: `EnumVariant` can be a struct or not, but how do we know that?
                    NameKind::Struct |
                    NameKind::EnumVariant { .. } => Ok(()),
                    k => Err(not_struct_error(path, k.into())),
                },
                NameOrigin::External => unreachable!(),
            }
        }

        match expr {
            Expr::Path(p) => match check_path(p) {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.errors.push(e);
                    Err(())
                },
            },
            Expr::Number { .. } |
            Expr::String { .. } |
            Expr::Char { .. } |
            Expr::Byte { .. } => Ok(()),
            Expr::Call { func, args, .. } => {
                let mut has_error = false;

                if let Err(()) = self.check_expr_path(func) {
                    has_error = true;
                }

                for CallArg { arg, .. } in args.iter() {
                    if let Err(()) = self.check_expr_path(arg) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::Tuple { elements, .. } |
            Expr::List { elements, .. } => {
                let mut has_error = false;

                for element in elements.iter() {
                    if let Err(()) = self.check_expr_path(element) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::StructInit { constructor, fields, .. } => {
                let mut has_error = false;

                if let Err(e) = check_struct_path(constructor) {
                    self.errors.push(e);
                    has_error = true;
                }

                for StructInitField { value, .. } in fields.iter() {
                    if let Err(()) = self.check_expr_path(value) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Expr::Field { lhs: hs, .. } |
            Expr::PrefixOp { rhs: hs, .. } |
            Expr::PostfixOp { lhs: hs, .. } => self.check_expr_path(hs),
            Expr::FieldUpdate { lhs, rhs, .. } |
            Expr::InfixOp { lhs, rhs, .. } => match (
                self.check_expr_path(lhs),
                self.check_expr_path(rhs),
            ) {
                (Ok(()), Ok(())) => Ok(()),
                _ => Err(()),
            },
        }
    }
}

fn not_expr_error(path: &Path, kind: NotExprBut) -> Error {
    Error {
        kind: ErrorKind::NotExpr {},
    }
}
