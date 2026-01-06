use sodigy_error::{Error, Warning};
use sodigy_mir::Type;
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Session {
    pub types: HashMap<Span, Type>,
    pub generic_instances: HashMap<(Span, Span), Type>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}
