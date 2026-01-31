use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{Expr, Path};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::RenderableSpan;

impl Session {
    pub fn resolve_poly(&mut self) -> Result<(), ()> {
        let mut has_error = false;

        for (mut path, impl_span) in self.poly_impls.clone().into_iter() {
            if let Err(()) = self.resolve_expr(&mut path) {
                has_error = true;
                continue;
            }

            if let Err(()) = self.check_expr_path(&path) {
                has_error = true;
                continue;
            }

            match path {
                Expr::Path(Path { id, fields, types }) if fields.is_empty() && types[0].is_none() => match self.polys.get_mut(&id.def_span) {
                    Some(poly) => {
                        poly.impls.push(impl_span);
                    },
                    None => {
                        let is_func = match id.origin {
                            NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => kind == NameKind::Func,
                            _ => false,
                        };

                        self.errors.push(Error {
                            kind: ErrorKind::NotPolyGeneric { id: Some(id) },
                            spans: vec![
                                RenderableSpan {
                                    span: id.span,
                                    auxiliary: false,
                                    note: Some(String::from("This is not a poly generic function.")),
                                },
                                RenderableSpan {
                                    span: id.def_span,
                                    auxiliary: true,
                                    note: Some(format!(
                                        "`{}` is defined here.",
                                        id.id.unintern_or_default(&self.intermediate_dir),
                                    )),
                                },
                            ],
                            note: Some(
                                if is_func {
                                    format!(
                                        "Use `#[poly]` to make `{}` a poly generic function.",
                                        id.id.unintern_or_default(&self.intermediate_dir),
                                    )
                                } else {
                                    format!(
                                        "`{}` is not even a function. Only a function can be a poly generic function.",
                                        id.id.unintern_or_default(&self.intermediate_dir),
                                    )
                                }
                            ),
                        });
                        has_error = true;
                    },
                },
                _ => {
                    self.errors.push(Error {
                        kind: ErrorKind::NotPolyGeneric { id: None },
                        spans: vec![
                            RenderableSpan {
                                span: path.error_span_wide(),
                                auxiliary: false,
                                note: Some(String::from("This is not a poly generic function.")),
                            },
                        ],
                        note: Some(String::from("Only a function can be a poly generic.")),
                    });
                    has_error = true;
                },
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }
}
