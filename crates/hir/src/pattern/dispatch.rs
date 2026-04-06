use super::{Pattern, PatternKind};
use sodigy_span::Span;
use std::collections::HashMap;

impl Pattern {
    pub fn dispatch(&mut self, dispatch_map: &HashMap<Span, Span>) {
        self.kind.dispatch(dispatch_map);
    }
}

impl PatternKind {
    pub fn dispatch(&mut self, dispatch_map: &HashMap<Span, Span>) {
        match self {
            PatternKind::Path(p) => match dispatch_map.get(&p.id.span) {
                Some(new_def_span) => {
                    assert!(p.fields.is_empty());
                    p.id.def_span = new_def_span.clone();
                    p.dotfish = vec![None];
                },
                None => {},
            },
            PatternKind::Constant(_) |
            PatternKind::NameBinding { .. } |
            PatternKind::Regex { .. } |
            PatternKind::Wildcard(_) => {},
            PatternKind::Struct { r#struct, fields, .. } => {
                match dispatch_map.get(&r#struct.id.span) {
                    Some(new_def_span) => {
                        assert!(r#struct.fields.is_empty());
                        r#struct.id.def_span = new_def_span.clone();
                        r#struct.dotfish = vec![None];
                    },
                    None => {},
                }

                for field in fields.iter_mut() {
                    field.pattern.dispatch(dispatch_map);
                }
            },
            PatternKind::TupleStruct { r#struct, elements, .. } => {
                match dispatch_map.get(&r#struct.id.span) {
                    Some(new_def_span) => {
                        assert!(r#struct.fields.is_empty());
                        r#struct.id.def_span = new_def_span.clone();
                        r#struct.dotfish = vec![None];
                    },
                    None => {},
                }

                for element in elements.iter_mut() {
                    element.dispatch(dispatch_map);
                }
            },
            PatternKind::Tuple { elements, .. } |
            PatternKind::List { elements, .. } => {
                for element in elements.iter_mut() {
                    element.dispatch(dispatch_map);
                }
            },
            PatternKind::Range { lhs, rhs, .. } => {
                if let Some(lhs) = lhs {
                    lhs.dispatch(dispatch_map);
                }

                if let Some(rhs) = rhs {
                    rhs.dispatch(dispatch_map);
                }
            },
            PatternKind::Or { lhs, rhs, .. } => {
                lhs.dispatch(dispatch_map);
                rhs.dispatch(dispatch_map);
            },
        }
    }
}
