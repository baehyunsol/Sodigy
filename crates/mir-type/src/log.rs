use crate::ErrorContext;
use sodigy_mir::Type;
use sodigy_span::Span;
use std::collections::HashMap;

pub enum TypeLog {
    // either `expected_type` or `subtype` is a type var
    SolveSubtype {
        expected_type: Type,
        subtype: Type,
        expected_span: Option<Span>,
        subtype_span: Option<Span>,
        context: ErrorContext,
    },
    Dispatch {
        call: Span,
        def: Span,
        generics: Vec<(Span, Type)>,
    },
    NeverType(Type),
}
