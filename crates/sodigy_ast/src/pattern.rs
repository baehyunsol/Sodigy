use crate::{
    IdentWithSpan,
    TypeDef,
};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_span::SpanRange;

#[derive(Clone)]
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

    pub fn get_name_bindings(&self) -> Vec<IdentWithSpan> {
        let mut bindings = vec![];

        match &self.kind {
            PatternKind::Identifier(_)
            | PatternKind::Number(_)
            | PatternKind::Char(_)
            | PatternKind::Path(_)
            | PatternKind::Wildcard
            | PatternKind::Shorthand => { /* nop */ },
            PatternKind::Binding(name) => {
                bindings.push(IdentWithSpan::new(*name, self.span));
            },
            PatternKind::Range { from, to, .. } => {
                for binding in from.as_ref().map(|pat| pat.get_name_bindings()).unwrap_or(vec![]) {
                    bindings.push(binding);
                }

                for binding in to.as_ref().map(|pat| pat.get_name_bindings()).unwrap_or(vec![]) {
                    bindings.push(binding);
                }
            },
            PatternKind::Tuple(patterns)
            | PatternKind::Slice(patterns)
            | PatternKind::TupleStruct {
                fields: patterns,
                ..
            } => {
                for pattern in patterns.iter() {
                    for binding in pattern.get_name_bindings() {
                        bindings.push(binding);
                    }
                }
            },
            PatternKind::Struct {
                fields, ..
            } => {
                for PatField { value, .. } in fields.iter() {
                    for binding in value.get_name_bindings() {
                        bindings.push(binding);
                    }
                }
            },
            PatternKind::Or(left, right) => {
                for binding in left.get_name_bindings() {
                    bindings.push(binding);
                }

                for binding in right.get_name_bindings() {
                    bindings.push(binding);
                }
            },
        }

        if let Some(id) = &self.bind {
            bindings.push(*id);
        }

        bindings
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
}

#[derive(Clone)]
pub enum PatternKind {
    Identifier(InternedString),
    Number(InternedNumeric),
    Char(char),
    Binding(InternedString),

    // path.len() > 1
    Path(Vec<IdentWithSpan>),

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
    Slice(Vec<Pattern>),

    // Foo { x: $x, y: $y, .. }
    Struct {
        struct_name: Vec<IdentWithSpan>,  // Foo
        fields: Vec<PatField>,
        has_shorthand: bool,
    },

    // Foo($a, .., $b)
    TupleStruct {
        name: Vec<IdentWithSpan>,
        fields: Vec<Pattern>,
    },

    Wildcard,   // _
    Shorthand,  // ..

    // will later be converted to Vec<Pattern>
    Or(Box<Pattern>, Box<Pattern>),
}

#[derive(Clone)]
pub struct PatField {
    name: IdentWithSpan,
    value: Pattern,
}
