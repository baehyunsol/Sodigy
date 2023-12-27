use crate::{
    DottedNames,
    IdentWithSpan,
    TypeDef,
};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_span::SpanRange;

mod fmt;
mod parse;

pub(crate) use parse::parse_pattern;

#[derive(Clone, Debug)]
pub struct Pattern {
    pub kind: PatternKind,
    pub ty: Option<TypeDef>,
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

    pub fn dummy_wildcard() -> Self {
        Pattern {
            kind: PatternKind::Wildcard,
            ty: None,
            span: SpanRange::dummy(0x130cd8bc),
            bind: None,
        }
    }

    pub fn is_wildcard(&self) -> bool {
        self.kind.is_wildcard()
    }

    pub fn is_shorthand(&self) -> bool {
        self.kind.is_shorthand()
    }
}

#[derive(Clone, Debug)]
pub enum PatternKind {
    Identifier(InternedString),
    Number {
        num: InternedNumeric,

        // all the numbers in expr are positive: `-` is an operator
        // but in patterns, we need this field because there's no `-` operator
        is_negative: bool,
    },
    Char(char),
    Binding(InternedString),

    // path.len() > 1
    Path(DottedNames),

    // 3..4
    // ..4
    // 3..
    // 'a'..~'z'
    Range {
        // either `from` or `to` has to be `Some(_)`
        // an end must either be
        // Ident, Number, Char, 
        from: Option<Box<Pattern>>,
        to: Option<Box<Pattern>>,
        inclusive: bool,
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
}

#[derive(Clone, Debug)]
pub struct PatField {
    pub name: IdentWithSpan,
    pub pattern: Pattern,
}
