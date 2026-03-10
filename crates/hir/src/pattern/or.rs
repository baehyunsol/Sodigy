use crate::{
    Pattern,
    PatternKind,
};
use sodigy_error::{Warning, WarningKind};
use sodigy_span::{RenderableSpan, Span};
use sodigy_token::Constant;

pub enum PatternSplit<'p> {
    NoSplit(&'p Pattern),
    Split(Vec<(Pattern, u32)>),
}

impl Pattern {
    // pattern: `([], _) | (_, [])`
    // -> splitted
    //
    // pattern: `[1 | 2 | 3, _]`
    // -> not splitted
    // -> post-mir can handle this case!
    // -> we can even optimize it to `[1..4, _]`
    //
    // pattern: `[1 | 4 | 7, _]`
    // -> not splitted
    // -> post-mir can handle this case, I believe...
    //
    // pattern: `[1 | Expr::Constant(_), _]`
    // -> this is an error, but this function will ignore all the errors
    //
    // pattern: `[1 | _, _]`
    // -> this is already filtered out
    //
    // pattern: `Expr::Constant(_) | Expr::If(_) | Expr::Call { .. }`
    // -> not splitted
    // -> post-mir can handle if no patterns have payloads
    //
    // pattern: `Expr::Constant(Constant::String { .. }) | Expr::If(_) | Expr::Call { .. }`
    // -> splitted, but splitted into 2 parts, not 3 parts
    // -> we don't have to split `If(_)` and `Call { .. }`.
    pub fn split_or_patterns<'p>(&'p self) -> PatternSplit<'p> {
        match &self.kind {
            PatternKind::Path(_) |
            PatternKind::Constant(_) |
            PatternKind::NameBinding { .. } |
            PatternKind::Regex { .. } |
            PatternKind::Wildcard(_) => PatternSplit::NoSplit(self),
            PatternKind::Struct { fields, .. } => todo!(),
            PatternKind::TupleStruct { elements, .. } |
            PatternKind::Tuple { elements, .. } |
            PatternKind::List { elements, .. } => {
                let splitted_elements: Vec<PatternSplit<'p>> = elements.iter().map(|element| element.split_or_patterns()).collect();

                if splitted_elements.iter().all(|e| matches!(e, PatternSplit::NoSplit(_))) {
                    PatternSplit::NoSplit(self)
                }

                else {
                    todo!()
                }
            },

            // There can't be or-patterns in lhs/rhs of a range-pattern.
            PatternKind::Range { .. } => PatternSplit::NoSplit(self),

            PatternKind::Or { lhs, rhs, .. } => {
                let mut operands = vec![];
                flatten_or_pattern(lhs, &mut operands);
                flatten_or_pattern(rhs, &mut operands);
                let mut can_be_grouped = vec![];
                let mut cannot_be_grouped = vec![];

                for operand in operands.iter() {
                    if operand.can_be_grouped_in_or_pattern() {
                        can_be_grouped.push(*operand);
                    }

                    else {
                        cannot_be_grouped.push(*operand);
                    }
                }

                if cannot_be_grouped.is_empty() {
                    PatternSplit::NoSplit(self)
                }

                else {
                    let mut splitted_patterns = Vec::with_capacity(cannot_be_grouped.len() + if can_be_grouped.is_empty() { 0 } else { 1 });

                    if !can_be_grouped.is_empty() {
                        splitted_patterns.push((into_or_pattern(&can_be_grouped), 0));
                    }

                    for pattern in cannot_be_grouped.iter() {
                        splitted_patterns.push(((*pattern).clone(), splitted_patterns.len() as u32));
                    }

                    PatternSplit::Split(splitted_patterns)
                }
            },
        }
    }

    fn can_be_grouped_in_or_pattern(&self) -> bool {
        match &self.kind {
            PatternKind::Path(_) => true,
            PatternKind::Constant(Constant::String { .. }) => false,
            PatternKind::Constant(_) => true,

            // These are supposed to be filtered out.
            PatternKind::NameBinding { .. } => false,
            PatternKind::Wildcard(_) => false,

            PatternKind::Regex { .. } => false,

            // an enum variant without payload must be grouped!
            PatternKind::Struct { .. } |
            PatternKind::TupleStruct { .. } => todo!(),

            PatternKind::Tuple { .. } => false,
            PatternKind::List { .. } => false,
            PatternKind::Range { .. } => true,
            PatternKind::Or { .. } => unreachable!(),
        }
    }
}

pub fn unreachable_or_pattern(reachable: &Pattern, unreachable: &Pattern) -> Warning {
    Warning {
        kind: WarningKind::UnreachableOrPattern,
        spans: vec![
            RenderableSpan {
                span: reachable.error_span_wide(),
                auxiliary: false,
                note: Some(String::from("This matches everything.")),
            },
            RenderableSpan {
                span: unreachable.error_span_wide(),
                auxiliary: true,
                note: Some(String::from("This pattern is meaningless.")),
            },
        ],
        note: None,
    }
}

fn flatten_or_pattern<'p>(pattern: &'p Pattern, result: &mut Vec<&'p Pattern>) {
    match &pattern.kind {
        PatternKind::Or { lhs, rhs, .. } => {
            flatten_or_pattern(lhs, result);
            flatten_or_pattern(rhs, result);
        },
        _ => {
            result.push(pattern);
        },
    }
}

fn into_or_pattern(patterns: &[&Pattern]) -> Pattern {
    match patterns.len() {
        0 => unreachable!(),
        1 => patterns[0].clone(),
        2.. => Pattern {
            name: None,
            name_span: None,
            kind: PatternKind::Or {
                lhs: Box::new(patterns[0].clone()),
                rhs: Box::new(into_or_pattern(&patterns[1..])),
                op_span: Span::None,
            },
        },
    }
}
