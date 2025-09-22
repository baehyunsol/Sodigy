use sodigy_error::{Error, ErrorKind};
use sodigy_file::File;
use sodigy_lex::LexSession;
use sodigy_span::Span;
use sodigy_token::{Token, TokenKind};

mod block;
mod decl;
mod expr;

pub use block::Block;
pub use decl::Decl;
pub use expr::Expr;

/// Actually, it's a `BlockParseSession` because a file of Sodigy is a block.
/// If there's a block inside a block, you have to create another session.
pub struct ParseSession<'t> {
    file: File,
    tokens: &'t [Token],
    cursor: usize,
    errors: Vec<Error>,
    decls: Vec<Decl>,
    value: Option<Expr>,
}

impl<'t> ParseSession<'t> {
    pub fn from_lex_session(s: &'t LexSession) -> ParseSession<'t> {
        ParseSession {
            file: s.file,
            tokens: &s.tokens,
            cursor: 0,
            errors: s.errors.clone(),
            decls: vec![],
            value: None,
        }
    }

    pub fn parse(&mut self) -> Result<Block, ()> {
        self.parse_block();
        todo!()
    }

    pub fn match_and_step(&mut self, token: TokenKind) -> Result<&'t Token, ()> {
        match self.tokens.get(self.cursor) {
            Some(t) if t.kind.matches(&token) => {
                self.cursor += 1;
                Ok(t)
            },
            Some(t) => {
                self.errors.push(Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: (&token).into(),
                        got: (&t.kind).into(),
                    },
                    span: t.span,
                });
                return Err(());
            },
            None => {
                self.errors.push(Error {
                    kind: ErrorKind::UnexpectedEof {
                        expected: (&token).into(),
                    },
                    span: Span::eof(self.file),
                });
                return Err(());
            },
        }
    }
}
