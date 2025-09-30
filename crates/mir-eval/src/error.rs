use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub enum Error {
    UndefinedName(InternedString),
    IndexError(i64),
}
