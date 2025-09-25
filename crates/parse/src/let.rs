use crate::{Decorator, DocComment, Expr, Tokens};
use sodigy_error::Error;
use sodigy_keyword::Keyword;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Expr>,
    pub value: Expr,
    pub doc_comment: Option<DocComment>,
    pub decorators: Vec<Decorator>,
}

impl<'t> Tokens<'t> {
    pub fn parse_let(&mut self) -> Result<Let, Vec<Error>> {
        self.match_and_pop(TokenKind::Keyword(Keyword::Let))?;
        let (name, name_span) = self.pop_name_and_span()?;

        let r#type = match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Colon), ..}) => {
                self.cursor += 1;
                Some(self.parse_expr()?)
            },
            _ => None,
        };
        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;
        let value = self.parse_expr()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Let {
            name,
            name_span,
            r#type,
            value,

            // Its parent will set these fields.
            doc_comment: None,
            decorators: vec![],
        })
    }
}
