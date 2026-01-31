use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{
    Path,
    Type,
};
use sodigy_name_analysis::{
    IdentWithOrigin,
    NameKind,
    NameOrigin,
};
use sodigy_parse::Field;
use sodigy_span::Span;

impl Session {
    // If there are complex aliases (alias of alias of alias of ...), this function
    // doesn't fully solve the path.
    // `resolve_alias` solves this issue by calling `resolve_use` and `resolve_type` over and
    // over until there's no more paths to solve. `resolve_alias` tracks whether it founds new
    // aliases or not with `log` parameter.
    //
    // The other resolvers (`resolve_expr`, `resolve_pattern`, ...) don't have such problem because
    // `resolve_alias` removes all the complex aliases.
    pub fn resolve_path(
        &mut self,

        // `Path::types` is for dotfish operators, which are only for expressions.
        // `Type::Param` acts like dotfish operators, but are not recorded in `Path::types`,
        // so `resolve_path` takes an extra parameter: `type_args`.
        // `type_args` is only for `Type::Param`.
        //
        // `std.result.Result<Int, Int>`
        // ->
        // path = { id: "std", fields: ["result", "Result"], types: [None, None, None] }
        // type_args = Some(["Int", "Int"])
        //
        // `x.convert.<Int>`
        // ->
        // path = { id: "x", fields: ["convert"], types: [None, Some("Int")] }
        // type_args = None
        path: &mut Path,
        type_args: Option<&[Type]>,

        log: &mut Vec<Span>,
    ) -> Result<(), ()> {
        // We have path `a.b.c`, and there's an alias `use x.y.z as a;`.
        // The path has to be resolved to `x.y.z.b.c`.
        // The resolved path keeps span and id of `a`, for better error messages.
        match self.name_aliases.get(&path.id.def_span) {
            Some(alias) => {
                log.push(path.id.span);
                log.push(path.id.def_span);
                path.id = IdentWithOrigin {
                    def_span: alias.path.id.def_span,
                    origin: alias.path.id.origin,
                    ..path.id
                };
                let alias_fields = alias.path.fields.iter().map(
                    |field| match field {
                        Field::Name { name, .. } => Field::Name {
                            name: *name,
                            name_span: path.id.span,
                            dot_span: path.id.span,
                            is_from_alias: true,
                        },
                        _ => unreachable!(),
                    }
                ).collect::<Vec<_>>();
                path.fields = vec![
                    alias_fields,
                    path.fields.to_vec(),
                ].concat();
                path.types = vec![
                    alias.path.types.clone(),
                    path.types[1..].to_vec(),
                ].concat();
            },
            None => {},
        }

        match self.type_aliases.get(&path.id.def_span) {
            Some(alias) => {
                log.push(path.id.span);
                log.push(path.id.def_span);
                let generic_args = match path.types.last() {
                    Some(Some(types)) => types.to_vec(),
                    _ => match type_args {
                        Some(types) => types.to_vec(),
                        None => vec![],
                    },
                };

                if generic_args.len() != alias.generics.len() {
                    // This is a compile error
                    // path: MyResult.<X>
                    // alias: type MyResult<T, E> = Result<T, E>;
                    todo!()
                }

                else {
                    let (alias_path, alias_generic_args) = match &alias.r#type {
                        Type::Path(path) => (path, None),
                        Type::Param { constructor, args, .. } => (constructor, Some(args.to_vec())),

                        // This is tricky:
                        // `type Tuple3<T> = (T, T, T);`
                        // `let x: Tuple3<Int> = { ... };`
                        // ->
                        // We called `resolve_path(Tuple3<Int>)`.
                        // There's no way we can represent `(Int, Int, Int)` with `Path`.
                        _ => todo!(),
                    };

                    path.id = IdentWithOrigin {
                        def_span: alias_path.id.def_span,
                        origin: alias_path.id.origin,
                        ..path.id
                    };
                    path.fields = alias_path.fields.iter().map(
                        |field| match field {
                            Field::Name { name, .. } => Field::Name {
                                name: *name,
                                name_span: path.id.span,
                                dot_span: path.id.span,
                                is_from_alias: true,
                            },
                            _ => unreachable!(),
                        }
                    ).collect();
                    path.types = alias_path.types.clone();
                    *path.types.last_mut().unwrap() = alias_generic_args;

                    if !generic_args.is_empty() {
                        // apply generic args
                        // For example, if `path` is `MyResult.<Int, ()>` and
                        // `alias` is `type MyResult<T, E> = Result<T, E>;`,
                        // we have to replace `<T, E>` in `Result<T, E>` with `<Int, ()>`.
                        todo!()
                    }
                }
            },
            None => {},
        }

        if let Some(field) = path.fields.get(0) {
            match self.item_name_map.get(&path.id.def_span) {
                Some((kind @ (NameKind::Module | NameKind::Enum), items)) => {
                    let (field_name, field_span) = (field.unwrap_name(), field.unwrap_name_span());

                    match items.get(&field_name) {
                        Some((item, item_kind)) => {
                            log.push(path.id.span);
                            log.push(path.id.def_span);
                            let new_id = IdentWithOrigin {
                                id: field_name,
                                span: field_span,
                                origin: NameOrigin::Foreign { kind: *item_kind },
                                def_span: *item,
                            };

                            if path.fields.len() == 1 {
                                *path = Path {
                                    id: new_id,
                                    fields: vec![],
                                    types: vec![None],
                                };
                                Ok(())
                            }

                            else {
                                *path = Path {
                                    id: new_id,
                                    fields: path.fields[1..].to_vec(),
                                    types: vec![
                                        vec![None],
                                        path.types[2..].to_vec(),
                                    ].concat(),
                                };
                                self.resolve_path(path, None, log)
                            }
                        },
                        None => {
                            let error_message = match kind {
                                NameKind::Module => format!(
                                    "Module `{}` doesn't have an item named `{}`.",
                                    path.id.id.unintern_or_default(&self.intermediate_dir),
                                    field_name.unintern_or_default(&self.intermediate_dir),
                                ),
                                NameKind::Enum => format!(
                                    "Enum `{}` doesn't have a variant named `{}`.",
                                    path.id.id.unintern_or_default(&self.intermediate_dir),
                                    field_name.unintern_or_default(&self.intermediate_dir),
                                ),
                                _ => unreachable!(),
                            };
                            self.errors.push(Error {
                                kind: ErrorKind::UndefinedName(field_name),
                                spans: field_span.simple_error(),
                                note: Some(error_message),
                            });
                            Err(())
                        },
                    }
                },
                Some((_, _)) => unreachable!(),
                None => Ok(()),
            }
        }

        else {
            Ok(())
        }
    }
}
