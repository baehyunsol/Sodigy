use sodigy_span::Span;

pub struct Warning {
    kind: WarningKind,
    span: Span,
}

pub enum WarningKind {}
