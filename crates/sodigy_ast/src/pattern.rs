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
            PatternKind::Or(left, right) => {
                left.get_name_bindings(buffer);
                right.get_name_bindings(buffer);
            },
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
            kind: PatternKind::Or(Box::new(pat1), Box::new(pat2)),
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

    // will later be converted to Vec<Pattern>
    Or(Box<Pattern>, Box<Pattern>),
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
