use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{
    Alias,
    Generic,
    Path,
    Type,
    Use,
};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::{RenderableSpan, Span};
use std::collections::{HashMap, HashSet};

// TODO: make it configurable
const ALIAS_RESOLVE_RECURSION_LIMIT: usize = 64;

impl Session {
    // Aliases might be nested. e.g.
    // `type x = foo;`
    // `use x as y;`
    // `use y as z;`
    //
    // Then, we have to resolve the above aliases to
    // `type x = foo;`
    // `use foo as y;`
    // `use foo as z;`
    //
    // We have to do this before resolving aliases in expressions and type annotations.
    // We have to do this globally.
    // Also, there can be an infinite loop, so we have to set some kinda recursion limit.
    pub fn resolve_alias(&mut self) -> Result<(), ()> {
        let mut nested_name_aliases = HashMap::new();
        let mut nested_type_aliases = HashMap::new();
        let mut name_aliases_to_type_aliases = vec![];
        let mut suspicious_spans = vec![];
        let mut has_error = false;

        for i in 0..(ALIAS_RESOLVE_RECURSION_LIMIT + 1) {
            let mut emergency_escape = false;

            for (name_span, mut alias) in self.type_aliases.clone().into_iter() {
                let mut alias_log = vec![];

                if let Err(()) = self.resolve_type(&mut alias.r#type, &mut alias_log) {
                    has_error = true;
                }

                if !alias_log.is_empty() {
                    if i == ALIAS_RESOLVE_RECURSION_LIMIT {
                        suspicious_spans.push(name_span);
                        suspicious_spans.extend(alias_log);
                    }

                    nested_type_aliases.insert(name_span, alias.r#type);
                }
            }

            for (name_span, mut r#use) in self.name_aliases.clone().into_iter() {
                let mut alias_log = vec![];

                if let Err(()) = self.resolve_use(&mut r#use, &mut name_aliases_to_type_aliases, &mut alias_log) {
                    has_error = true;
                }

                if !alias_log.is_empty() {
                    // `use x.a.b as y;`
                    // `use y.c.d as x;`
                    // -> When you resolve this `n` times, the length of the
                    //    field will be `2^n`.
                    if r#use.path.fields.len() > 1024 {
                        suspicious_spans = alias_log;
                        suspicious_spans.push(name_span);
                        emergency_escape = true;
                    }

                    else if i == ALIAS_RESOLVE_RECURSION_LIMIT {
                        suspicious_spans.push(name_span);
                        suspicious_spans.extend(alias_log);
                    }

                    nested_name_aliases.insert(name_span, r#use);
                }
            }

            if i == ALIAS_RESOLVE_RECURSION_LIMIT || emergency_escape {
                suspicious_spans = suspicious_spans.into_iter().collect::<HashSet<_>>().into_iter().collect();
                self.errors.push(Error {
                    kind: ErrorKind::AliasResolveRecursionLimitReached,
                    spans: suspicious_spans.iter().map(
                        |span| RenderableSpan {
                            span: *span,
                            auxiliary: false,
                            note: None,
                        }
                    ).collect(),
                    note: Some(String::from("It seems like these names are aliases of each other.")),
                });
                return Err(());
            }

            else if has_error {
                return Err(());
            }

            else if !nested_name_aliases.is_empty() || !nested_type_aliases.is_empty() || !name_aliases_to_type_aliases.is_empty() {
                for (name_span, r#use) in nested_name_aliases.drain() {
                    self.name_aliases.insert(name_span, r#use);
                }

                for (name_span, alias) in nested_type_aliases.drain() {
                    let old_alias = self.type_aliases.get_mut(&name_span).unwrap();
                    old_alias.r#type = alias;
                }

                for (name_span, type_alias) in name_aliases_to_type_aliases.drain(..) {
                    self.name_aliases.remove(&name_span);
                    self.type_aliases.insert(name_span, type_alias);
                }
            }

            else {
                break;
            }
        }

        Ok(())
    }

    // If `x` in `use x.y.z as w;` is an alias, it resolves `x`.
    // If `x` is a module in `use x.y as w;`, it finds the def_span of `y` and
    // replaces the alias with `use y as w;`.
    //
    // There may be multiple levels of aliases in `use`. This function only resolves
    // one level of alias. `resolve_alias` will call this function multiple times
    // until all the aliases are resolved (or an `AliasResolveRecursionLimitReached` error).
    //
    // `log` does 2 things:
    //     1. It tells whether the function has resolved something. If `log` is not empty, something has happened.
    //     2. When the solver throws `AliasResolveRecursionLimitReached` error, it looks at `log` to create an error message.
    pub fn resolve_use(
        &mut self,
        r#use: &mut Use,
        name_aliases_to_type_aliases: &mut Vec<(Span, Alias)>,
        log: &mut Vec<Span>,
    ) -> Result<(), ()> {
        // There's a special case that `resolve_path` cannot handle.
        // Think there's `use x as w;` and `type x<T> = Option<T>;`.
        // Then we want to lower the use statement to `type w<T> = Option<T>;`.
        if r#use.path.fields.is_empty() && let Some(type_alias) = self.type_aliases.get(&r#use.path.id.def_span) {
            name_aliases_to_type_aliases.push((
                r#use.name_span,
                Alias {
                    visibility: r#use.visibility.clone(),
                    keyword_span: r#use.keyword_span,
                    name: r#use.name,
                    name_span: r#use.name_span,
                    generics: type_alias.generics.iter().map(
                        |generic| Generic {
                            name: generic.name,
                            name_span: r#use.path.id.span,
                            // TODO: we need an extra field that it's from an alias
                        }
                    ).collect(),
                    generic_group_span: Some(r#use.path.id.span),
                    r#type: type_alias.r#type.clone(),
                    foreign_names: type_alias.foreign_names.clone(),
                },
            ));
            Ok(())
        }

        else {
            self.resolve_path(&mut r#use.path, None, log)?;

            // In case of `use x as y; type x = Foo<Int>;`, we have to
            // resolve the `use` statement to a type alias: `type y = Foo<Int>;`
            if let Some(Some(types)) = r#use.path.types.last() {
                let types = types.clone();
                *r#use.path.types.last_mut().unwrap() = None;

                name_aliases_to_type_aliases.push((
                    r#use.name_span,
                    Alias {
                        visibility: r#use.visibility.clone(),
                        keyword_span: r#use.keyword_span,
                        name: r#use.name,
                        name_span: r#use.name_span,
                        generics: vec![],
                        generic_group_span: None,
                        r#type: Type::Param {
                            constructor: r#use.path.clone(),
                            args: types,
                            group_span: Span::None,
                        },
                        foreign_names: HashMap::new(),
                    },
                ));
            }

            self.cannot_alias_local_names(&r#use.path)?;
            Ok(())
        }
    }

    // `fn foo(x) = { use x as y; y + 1 };` is illegal. `use` cannot alias
    // local values (inline let, func param, pattern name binding, ...).
    // It's necessary because
    //    1. When hir checks whether a lambda is a closure or not, it has to
    //       check whether there are captured local values or not.
    //    2. Hir checks `NameKind` to check whether a name is local or not.
    //       It assumes that `NameKind::Use` is not a local value.
    //    3. So, inter-hir has to guarantee that...
    //
    // It's not just about local vs global.
    // Locally defined enum and structs are okay to be used with `use`, because
    // they're not values!
    // For example, `fn foo(x) = { enum Foo = { ... }; use Foo as y; ... }` is okay!
    //
    // Generic params are not local values. If they're used in an expression, that's an
    // error and `check_type_annot_path` is responsible for catching that. If they're used
    // in a type annotation, that's not an error!
    fn cannot_alias_local_names(&mut self, path: &Path) -> Result<(), ()> {
        // If the alias is not fully resolved yet, this function does nothing and returns.
        // It'll be called again when the resolution is complete!
        let is_local_value = match path.id.origin {
            NameOrigin::FuncParam { .. } => Some(true),
            NameOrigin::GenericParam { .. } => Some(false),
            NameOrigin::Local { kind } |
            NameOrigin::Foreign { kind } => match kind {
                NameKind::Let { is_top_level } => Some(!is_top_level),

                // struct/enum/module are not values.
                // func/enum_variant are values, but they're global (Sodigy lifts inline funcs)
                NameKind::Func |
                NameKind::Struct |
                NameKind::Enum |
                NameKind::EnumVariant { .. } |
                NameKind::Module |
                NameKind::GenericParam => Some(false),

                // not resolved yet
                NameKind::Alias |
                NameKind::Use => None,

                NameKind::FuncParam |
                NameKind::PatternNameBind |
                NameKind::Pipeline => Some(true),
            },
            NameOrigin::External => None,
        };

        match is_local_value {
            Some(false) | None => Ok(()),
            Some(true) => {
                self.errors.push(Error {
                    kind: ErrorKind::CannotAliasLocalValue(path.id.id),
                    spans: vec![
                        RenderableSpan {
                            span: path.id.span,
                            auxiliary: false,
                            note: Some(String::from("This is a local value.")),
                        },
                        RenderableSpan {
                            span: path.id.def_span,
                            auxiliary: true,
                            note: Some(format!(
                                "`{}` is defined here.",
                                path.id.id.unintern_or_default(&self.intermediate_dir),
                            )),
                        },
                    ],
                    note: None,
                });
                Err(())
            },
        }
    }
}
