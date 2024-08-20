pub enum ArgKind {
    None,
    Path,

    // (NAME=PATH)+
    Library,
    String,
    Integer,

    Optional(Box<ArgKind>),
}

impl ArgKind {
    pub fn parse_single_token(&self, token: &Token) -> Result<Arg, ClapError> {
        match self {
            ArgKind::None
            | ArgKind::Library
            | ArgKind::Optional(_) => unreachable!(),
            ArgKind::Path => match String::from_utf8(&token.buffer) {
                Ok(s) => Ok(Arg::Path(s)),
                Err(e) => {},
            },
            ArgKind::String => match String::from_utf8(&token.buffer) {
                Ok(s) => Ok(Arg::String(s)),
                Err(e) => {},
            },
            ArgKind::Integer => match String::from_utf8(&token.buffer) {
                Ok(s) => match BigInt::from_string(&s) {
                    Ok(n) => match i64::try_from(n) {
                        Ok(n) => Ok(Arg::Integer(n)),
                        Err(e) => {},
                    },
                    Err(e) => {},
                },
                Err(e) => {},
            },
        }
    }
}

pub enum Arg {
    Path(String),
    String(String),
    Integer(i64),
    Library(HashMap<String, String>),
}
