use sodigy_number::InternedNumber;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub enum Value {
    Number(InternedNumber),
    List(Vec<Value>),
    Bool(bool),

    // You can use this span to find the definition of the functor
    // in `context.funcs`.
    Functor(Span),
}
