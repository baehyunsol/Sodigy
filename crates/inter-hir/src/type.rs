use crate::{Session, TypeStructExpr, not_x_but_y};
use sodigy_error::{Error, ErrorKind, NotXBut};
use sodigy_hir::{Path, Type};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::Span;

impl Session {
    // It resolves names in type annotations and type aliases.
    // See the comments in `resolve_use` for more information.
    pub fn resolve_type(
        &mut self,
        r#type: &mut Type,
        log: &mut Vec<Span>,
    ) -> Result<(), ()> {
        match r#type {
            Type::Path(path) => {
                self.resolve_path(path, None, log)?;

                if let Some(Some(types)) = path.types.last() {
                    let mut path = path.clone();
                    let types = types.clone();

                    *path.types.last_mut().unwrap() = None;
                    *r#type = Type::Param {
                        constructor: path,
                        args: types,
                        group_span: Span::None,
                    };
                }

                Ok(())
            },
            Type::Param { constructor, args, .. } => {
                let mut has_error = false;

                for arg in args.iter_mut() {
                    if let Err(()) = self.resolve_type(arg, log) {
                        has_error = true;
                    }
                }

                if !has_error {
                    if let Err(()) = self.resolve_path(constructor, Some(&args), log) {
                        has_error = true;
                    }

                    if let Some(Some(types)) = constructor.types.last() {
                        let types = types.clone();
                        *constructor.types.last_mut().unwrap() = None;
                        *args = types;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Type::Func { fn_constructor, params, r#return, .. } => {
                let mut has_error = false;

                if let Err(()) = self.resolve_path(fn_constructor, None, log) {
                    has_error = true;
                }

                if let Err(()) = self.resolve_type(r#return, log) {
                    has_error = true;
                }

                for param in params.iter_mut() {
                    if let Err(()) = self.resolve_type(param, log) {
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
            Type::Tuple { types, .. } => {
                let mut has_error = false;

                for r#type in types.iter_mut() {
                    if let Err(()) = self.resolve_type(r#type, log) {
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
            Type::Wildcard(_) | Type::Never(_) => Ok(()),
        }
    }

    pub fn check_type_annot_path(&mut self, r#type: &Type) -> Result<(), ()> {
        fn check_path(path: &Path, intermediate_dir: &str) -> Result<(), Error> {
            match path.id.origin {
                // What kinda error is this?
                _ if !path.fields.is_empty() => todo!(),

                // `parse::parse_type` will never generate this, neither inter-hir
                _ if path.types[0].is_some() => Err(Error {
                    kind: ErrorKind::InternalCompilerError { id: 636810 },
                    spans: path.error_span_wide().simple_error(),
                    note: None,
                }),
                NameOrigin::FuncParam { .. } => Err(not_x_but_y(path, TypeStructExpr::Type, NotXBut::Expr, intermediate_dir)),
                NameOrigin::GenericParam { .. } => Ok(()),
                NameOrigin::Local { kind } |
                NameOrigin::Foreign { kind } => match kind {
                    NameKind::Let { .. } |
                    NameKind::Func |
                    NameKind::EnumVariant { .. } |
                    NameKind::Module |
                    NameKind::FuncParam |
                    NameKind::PatternNameBind |
                    NameKind::Pipeline => Err(not_x_but_y(path, TypeStructExpr::Type, kind.into(), intermediate_dir)),
                    NameKind::Struct |
                    NameKind::Enum |
                    NameKind::GenericParam => Ok(()),

                    // inter-hir should have resolved it
                    NameKind::Alias | NameKind::Use => Err(Error {
                        kind: ErrorKind::InternalCompilerError { id: 636811 },
                        spans: path.error_span_wide().simple_error(),
                        note: None,
                    }),
                },
                // inter-hir should have resolved it
                NameOrigin::External => Err(Error {
                    kind: ErrorKind::InternalCompilerError { id: 636812 },
                    spans: path.error_span_wide().simple_error(),
                    note: None,
                }),
            }
        }

        match r#type {
            Type::Path(p) => match check_path(p, &self.intermediate_dir) {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.errors.push(e);
                    Err(())
                },
            },
            Type::Param { constructor, args, .. } => {
                let mut has_error = false;

                for arg in args.iter() {
                    if let Err(()) = self.check_type_annot_path(arg) {
                        has_error = true;
                    }
                }

                if let Err(e) = check_path(constructor, &self.intermediate_dir) {
                    self.errors.push(e);
                    has_error = true;
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Type::Tuple { types, .. } => {
                let mut has_error = false;

                for r#type in types.iter() {
                    if let Err(()) = self.check_type_annot_path(r#type) {
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
            Type::Func { fn_constructor, params, r#return, .. } => {
                let mut has_error = false;

                if let Err(e) = check_path(fn_constructor, &self.intermediate_dir) {
                    self.errors.push(e);
                    has_error = true;
                }

                for param in params.iter() {
                    if let Err(()) = self.check_type_annot_path(param) {
                        has_error = true;
                    }
                }

                if let Err(()) = self.check_type_annot_path(r#return) {
                    has_error = true;
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            Type::Wildcard(_) | Type::Never(_) => Ok(()),
        }
    }
}
