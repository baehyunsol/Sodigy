use crate::{
    DottedNames,
    IdentWithSpan,
    TypeDef,
    error::AstError,
    session::AstSession,
};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use std::collections::HashMap;

mod fmt;
mod parse;

pub(crate) use parse::parse_pattern_full;

#[derive(Clone, Debug)]
pub struct Pattern {
    pub kind: PatternKind,
    pub ty: Option<TypeDef>,

    // spans of `|` and `..` don't include lhs and rhs
    pub span: SpanRange,

    // binds the entire pattern to a name
    // the bound name must be followed by the pattern
    // the name is prefixed with `$`
    // e.g. `$tok @ Token { kind: $kind, .. }`
    pub bind: Option<IdentWithSpan>,
}

impl Pattern {
    pub fn set_ty(&mut self, ty: TypeDef) {
        self.ty = Some(ty);
    }

    pub fn set_bind(&mut self, bind: IdentWithSpan) {
        self.bind = Some(bind);
    }

    pub fn try_into_binding(&self) -> Option<IdentWithSpan> {
        match &self.kind {
            PatternKind::Binding(id) => Some(IdentWithSpan::new(*id, self.span)),
            _ => None,
        }
    }

    pub fn get_name_bindings(&self, buffer: &mut Vec<IdentWithSpan>) {
        match &self.kind {
            PatternKind::Identifier(_)
            | PatternKind::Number { .. }
            | PatternKind::Char(_)
            | PatternKind::String { .. }
            | PatternKind::Path(_)
            | PatternKind::Wildcard
            | PatternKind::Shorthand => { /* nop */ },
            PatternKind::Binding(name) => {
                buffer.push(IdentWithSpan::new(*name, self.span));
            },
            PatternKind::Range { from, to, .. } => {
                if let Some(pat) = from.as_ref() {
                    pat.get_name_bindings(buffer);
                }

                if let Some(pat) = to.as_ref() {
                    pat.get_name_bindings(buffer);
                }
            },
            PatternKind::Tuple(patterns)
            | PatternKind::List(patterns)
            | PatternKind::TupleStruct {
                fields: patterns,
                ..
            } => {
                for pattern in patterns.iter() {
                    pattern.get_name_bindings(buffer);
                }
            },
            PatternKind::Struct {
                fields, ..
            } => {
                for PatField { pattern, .. } in fields.iter() {
                    pattern.get_name_bindings(buffer);
                }
            },
            // 1. The result of this function is used to check name collisions.
            //    - TODO: it's not a good idea to restrict the usage of this function
            // 2. `($x, 0) | ($x, 1)` is not a collision but `($x, $x) | (0, 1)` is.
            // 3. There's another restriction on `or` patterns: each pattern must have the same set of names.
            //    - These conditions are checked by `check_names_in_or_patterns`.
            // 4. This function just assumes that there's no name errors in the `or` pattern.
            PatternKind::Or(patterns) => {
                let mut tmp_buffer = vec![];

                for pattern in patterns.iter() {
                    pattern.get_name_bindings(&mut tmp_buffer);
                }

                let deduplicated: HashMap<InternedString, SpanRange> = tmp_buffer.into_iter().map(
                    |name| (name.id(), *name.span())
                ).collect();

                for (id, span) in deduplicated.into_iter() {
                    buffer.push(IdentWithSpan::new(id, span));
                }
            },
            PatternKind::OrRaw(left, right) => unreachable!(),
        }

        if let Some(id) = &self.bind {
            // let's not push the same name twice
            if let PatternKind::Binding(name) = &self.kind {
                if id.id() != *name {
                    buffer.push(*id);
                }
            }

            else {
                buffer.push(*id);
            }
        }
    }

    pub fn bind_name(&mut self, name: IdentWithSpan) {
        self.bind = Some(name);
    }

    pub fn or(pat1: Self, pat2: Self) -> Self {
        let span = pat1.span.merge(pat2.span);

        Pattern {
            kind: PatternKind::OrRaw(Box::new(pat1), Box::new(pat2)),
            ty: None,
            span,
            bind: None,
        }
    }

    pub fn dummy_wildcard(span: SpanRange) -> Self {
        Pattern {
            kind: PatternKind::Wildcard,
            ty: None,
            span,
            bind: None,
        }
    }

    pub fn is_wildcard(&self) -> bool {
        self.kind.is_wildcard()
    }

    pub fn is_shorthand(&self) -> bool {
        self.kind.is_shorthand()
    }

    pub fn is_string(&self) -> bool {
        self.kind.is_string()
    }

    // if it has a type annot or a name binding,
    // it pushes an error to session and returns Err
    pub fn assert_no_type_and_no_binding(
        &self,
        session: &mut AstSession,
    ) -> Result<(), ()> {
        let mut has_error = false;

        if let Some(ty) = &self.ty {
            session.push_error(
                AstError::type_anno_not_allowed(ty.0.span)
            );

            has_error = true;
        }

        if let Some(bind) = &self.bind {
            session.push_error(
                AstError::name_binding_not_allowed(*bind.span())
            );

            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub enum PatternKind {
    Identifier(InternedString),
    Number(InternedNumeric),
    Char(char),
    String {
        content: InternedString,
        is_binary: bool,  // `b` prefix
    },
    Binding(InternedString),

    // path.len() > 1
    Path(DottedNames),

    // 3..4
    // ..4
    // 3..
    // 'a'..~'z'
    Range {
        // either `from` or `to` has to be `Some(_)`
        from: Option<Box<Pattern>>,
        to: Option<Box<Pattern>>,
        inclusive: bool,

        // either `from` or `to` is a string
        is_string: bool,
    },

    // ($a, .., $b)
    Tuple(Vec<Pattern>),

    // [$a, .., $b]
    List(Vec<Pattern>),

    // Foo { x: $x, y: $y, .. }
    Struct {
        struct_name: DottedNames,
        fields: Vec<PatField>,
        has_shorthand: bool,
    },

    // Foo($a, .., $b)
    TupleStruct {
        name: DottedNames,
        fields: Vec<Pattern>,
    },

    Wildcard,   // _
    Shorthand,  // ..

    // `|` operators are first lowered to `OrRaw` and then to `Or`.
    // `OrRaw` is only used as intermediate representation in `parse_pattern` functions
    // it's a compiler bug if this variant is found in other contexts
    OrRaw(Box<Pattern>, Box<Pattern>),

    // It's guaranteed to be non-recursive.
    Or(Vec<Pattern>),
}

impl PatternKind {
    pub fn is_wildcard(&self) -> bool {
        matches!(self, PatternKind::Wildcard)
    }

    pub fn is_shorthand(&self) -> bool {
        matches!(self, PatternKind::Shorthand)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, PatternKind::String { .. })
    }
}

#[derive(Clone, Debug)]
pub struct PatField {
    pub name: IdentWithSpan,
    pub pattern: Pattern,
}
