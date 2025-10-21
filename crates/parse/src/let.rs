use crate::{Attribute, Expr, Tokens, Type};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Keyword, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Let {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Type>,
    pub value: Expr,
    pub attribute: Attribute,
}

impl<'t> Tokens<'t> {
    pub fn parse_let(&mut self) -> Result<Vec<Let>, Vec<Error>> {
        let prev_cursor = self.cursor;

        match self.parse_let_simple() {
            Ok(r#let) => Ok(vec![r#let]),
            Err(_) => {
                self.cursor = prev_cursor;
                self.parse_let_multiple()
            },
        }
    }

    // Most `let` statements are in this form, so let's do some optimization.
    fn parse_let_simple(&mut self) -> Result<Let, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Let))?.span;
        let (name, name_span) = self.pop_name_and_span()?;

        let r#type = match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Colon), ..}) => {
                self.cursor += 1;
                Some(self.parse_type()?)
            },
            _ => None,
        };
        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;
        let value = self.parse_expr()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Let {
            keyword_span,
            name,
            name_span,
            r#type,
            value,
            attribute: Attribute::new(),
        })
    }

    // It destructures a pattern into multiple `let` statements.
    fn parse_let_multiple(&mut self) -> Result<Vec<Let>, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Let))?.span;
        let pattern = self.parse_full_pattern()?;
        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;
        let value = self.parse_expr()?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        // TODO: destructure the pattern into 1 or more `let` statements
        // TODO: make sure that the pattern is irrefutable
        // TODO: it might need type-checks (e.g. count the number of elements in tuple)
        //       we have to store the info somewhere, so that mir can check types
        todo!()
    }
}
