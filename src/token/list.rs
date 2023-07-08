use super::{Delimiter, Keyword, OpToken, Token, TokenKind};
use crate::err::{ExpectedToken, ParseError};
use crate::expr::{parse_expr, Expr, ExprKind, InfixOp, PostfixOp, PrefixOp};
use crate::parse::{parse_expr_exhaustive, split_list_by_comma};
use crate::session::InternedString;
use crate::span::Span;
use crate::stmt::{parse_arg_def, ArgDef};
use crate::value::{parse_block_expr, Value};

pub struct TokenList {
    pub data: Vec<Token>,
    cursor: usize,
}

impl TokenList {
    pub fn from_vec(data: Vec<Token>) -> Self {
        TokenList { data, cursor: 0 }
    }

    pub fn from_vec_box_token(data: Vec<Box<Token>>) -> Self {
        TokenList {
            data: data
                .into_iter()
                .map(|token| Box::leak(token).to_owned())
                .collect(),
            cursor: 0,
        }
    }

    pub fn is_eof(&self) -> bool {
        assert!(
            self.cursor <= self.data.len(),
            "Internal Compiler Error 9789F9F"
        );

        self.cursor >= self.data.len()
    }

    pub fn last_token(&self) -> Option<&Token> {
        self.data.last()
    }

    pub fn ends_with(&self, token_kind: TokenKind) -> bool {
        match self.data.last() {
            Some(Token { kind, .. }) if *kind == token_kind => true,
            _ => false,
        }
    }

    pub fn backward(&mut self) {
        self.cursor -= 1;
    }

    pub fn count_tokens_non_recursive(&self, kind: TokenKind) -> usize {
        let mut count = 0;

        for token in self.data[self.cursor..].iter() {
            if &token.kind == &kind {
                count += 1;
            }
        }

        count
    }

    pub fn append(&mut self, mut tokens: Vec<Token>) {
        self.data.append(&mut tokens);
    }

    fn contains(&self, kind: &TokenKind) -> bool {

        for token in self.data[self.cursor..].iter() {
            if &token.kind == kind {
                return true;
            }
        }

        false
    }

    // if the current token is `token`, it steps forward and returns true
    // it returns false otherwise
    // it's helpful if the borrow checker doesn't allow you to use `self.step`
    pub fn consume(&mut self, token: TokenKind) -> bool {
        match self.data.get(self.cursor) {
            Some(t) if t.kind == token => {
                self.cursor += 1;
                true
            }
            _ => false,
        }
    }

    pub fn consume_token_or_error(&mut self, token_kind: TokenKind) -> Result<(), ParseError> {
        match self.step() {
            Some(Token { kind, .. }) if *kind == token_kind => Ok(()),
            Some(Token { kind, span }) => Err(ParseError::tok(
                kind.clone(),
                *span,
                ExpectedToken::SpecificTokens(vec![token_kind]),
            )),
            None => Err(ParseError::eoe(
                Span::dummy(),
                ExpectedToken::SpecificTokens(vec![token_kind]),
            )),
        }
    }

    // `get_XXX` functions don't move the cursor
    // it returns None if the cursor is not pointing to the data

    pub fn get_curr_span(&self) -> Option<Span> {
        self.data.get(self.cursor).map(|t| t.span)
    }

    // `step_XXX` functions (including `step`)
    // if the current token is `XXX`, it returns `Some(XXX)` and steps the cursor forward
    // otherwise, it doesn't do anything and returns `None`
    // `step_XXX_strict` are like `step_XXX`, but returns `Err()` instead of `None`

    pub fn step_identifier_strict(&mut self) -> Result<InternedString, ParseError> {
        match self.step() {
            Some(Token { kind, .. }) if kind.is_identifier() => Ok(kind.unwrap_identifier()),
            Some(Token { kind, span }) => Err(ParseError::tok(
                kind.clone(),
                *span,
                ExpectedToken::SpecificTokens(vec![
                    TokenKind::dummy_identifier(),
                ]),
            )),
            None => Err(ParseError::eoe(
                Span::dummy(),
                ExpectedToken::SpecificTokens(vec![
                    TokenKind::dummy_identifier(),
                ]),
            ))
        }
    }

    pub fn step(&mut self) -> Option<&Token> {
        let result = self.data.get(self.cursor);

        if result.is_some() {
            self.cursor += 1;
        }

        result
    }

    pub fn step_func_def_args(&mut self) -> Option<Result<Vec<ArgDef>, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::List(Delimiter::Parenthesis, elements),
                span,
            }) => {
                let mut args = vec![];
                let mut args_tokens = TokenList::from_vec_box_token(elements.to_vec());

                while !args_tokens.is_eof() {
                    match parse_arg_def(&mut args_tokens) {
                        Ok(arg) => {
                            args.push(arg);
                        }
                        Err(e) => {
                            return Some(Err(e.set_span_of_eof(*span)));
                        }
                    }

                    match args_tokens.step() {
                        Some(Token {
                            kind: TokenKind::Operator(OpToken::Comma),
                            ..
                        }) => {}
                        Some(Token { kind, span }) => {
                            return Some(Err(ParseError::tok(
                                kind.clone(),
                                *span,
                                ExpectedToken::SpecificTokens(vec![
                                    TokenKind::comma(),
                                ]),
                            )));
                        }
                        None => {
                            break;
                        }
                    }
                }

                Some(Ok(args))
            }
            _ => None,
        }
    }

    // it returns `None` only when there's no token to parse
    pub fn step_type(&mut self) -> Option<Result<Expr, ParseError>> {
        match self.data.get(self.cursor) {
            Some(_) => Some(parse_expr(self, 0)),
            None => None,
        }
    }

    pub fn step_prefix_op(&mut self) -> Option<PrefixOp> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::Operator(op),
                ..
            }) => match op {
                OpToken::Sub | OpToken::Not => {
                    self.cursor += 1;
                    Some(op.into())
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub fn step_infix_op(&mut self) -> Option<InfixOp> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::Operator(op),
                ..
            }) => match op {
                OpToken::Add
                | OpToken::Sub
                | OpToken::Mul
                | OpToken::Div
                | OpToken::Rem
                | OpToken::Eq
                | OpToken::Gt
                | OpToken::Lt
                | OpToken::Ne
                | OpToken::Ge
                | OpToken::Le
                | OpToken::Dot
                | OpToken::DotDot
                | OpToken::Concat
                | OpToken::And
                | OpToken::AndAnd
                | OpToken::Or
                | OpToken::OrOr => {
                    self.cursor += 1;
                    Some(op.into())
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub fn step_postfix_op(&mut self) -> Option<PostfixOp> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::Operator(op),
                ..
            }) => match op {
                // `..` can either be infix or postfix, it depends on the context
                OpToken::DotDot => match self.data.get(self.cursor + 1) {
                    // postfix
                    None => {
                        self.cursor += 1;
                        Some(op.into())
                    }

                    Some(Token { kind, .. }) => match kind {
                        // TODO: `1..-2..-3` -> Range(Range(1, -2), -3) or Sub(Range(1), Range(2, -3))
                        //     -> it's not decided by the precedence, but by this function
                        TokenKind::Operator(next_op) => match next_op {
                            // postfix
                            OpToken::Comma => {
                                self.cursor += 1;
                                Some(op.into())
                            }

                            // infix
                            OpToken::Sub | OpToken::Not => None,

                            // TODO: not sure what to do
                            _ => {
                                self.cursor += 1;
                                Some(op.into())
                            }
                        },

                        // infix
                        _ => None,
                    },
                },
                _ => None,
            },
            _ => None,
        }
    }

    // Some(Err(_)) indicates that it's a branch_expr but the inner expr has a syntax error
    // if self.step() is `if`, it must return Some(_)
    pub fn step_branch_expr(&mut self) -> Option<Result<Expr, ParseError>> {
        let if_span = if let Some(s) = self.get_curr_span() {
            s
        } else {
            return None;
        };

        if self.consume(TokenKind::keyword_if()) {
            let cond = match parse_expr(self, 0) {
                Ok(expr) => expr,
                Err(e) => {
                    return Some(Err(e.set_span_of_eof(if_span)));
                }
            };

            let true_expr = match self.step() {
                Some(Token {
                    kind: TokenKind::List(Delimiter::Brace, elements),
                    span: true_expr_span,
                }) => match parse_block_expr(&mut TokenList::from_vec_box_token(elements.to_vec()))
                {
                    Ok(t) => Expr {
                        kind: t.block_to_expr_kind(),
                        span: *true_expr_span,
                    },
                    Err(e) => {
                        return Some(Err(e.set_span_of_eof(*true_expr_span)));
                    }
                },
                Some(Token { kind, span }) => {
                    return Some(Err(ParseError::tok(
                        kind.clone(),
                        *span,
                        ExpectedToken::SpecificTokens(vec![
                            TokenKind::List(Delimiter::Brace, vec![]),
                        ]),
                    )));
                }
                None => {
                    return Some(Err(ParseError::eoe(
                        if_span,
                        ExpectedToken::SpecificTokens(vec![
                            TokenKind::Operator(OpToken::OpeningCurlyBrace),
                        ]),
                    )));
                }
            };

            let else_span = self.get_curr_span();

            let false_expr = if self.consume(TokenKind::keyword_else()) {
                match self.step() {
                    Some(Token {
                        kind: TokenKind::Keyword(Keyword::If),
                        ..
                    }) => {
                        // `step_branch_expr` reads from `Keyword::If`
                        self.backward();

                        match self.step_branch_expr() {
                            Some(Ok(false_expr)) => false_expr,
                            Some(Err(e)) => {
                                return Some(Err(e));
                            }
                            None => unreachable!("Internal Compiler Error A453107"),
                        }
                    },
                    Some(Token {
                        kind: TokenKind::List(Delimiter::Brace, elements),
                        span: false_expr_span,
                    }) => match parse_block_expr(
                        &mut TokenList::from_vec_box_token(elements.to_vec())
                    ) {
                        Ok(t) => Expr {
                            kind: t.block_to_expr_kind(),
                            span: *false_expr_span,
                        },
                        Err(e) => {
                            return Some(Err(e.set_span_of_eof(*false_expr_span)));
                        }
                    }
                    Some(Token { kind, span }) => {
                        return Some(Err(ParseError::tok(
                            kind.clone(),
                            *span,
                            ExpectedToken::SpecificTokens(vec![
                                TokenKind::keyword_if(),
                                TokenKind::List(Delimiter::Brace, vec![]),
                            ]),
                        )))
                    }
                    None => {
                        return Some(Err(ParseError::eoe(
                            else_span.expect("Internal Compiler Error 26CED6F"),
                            ExpectedToken::SpecificTokens(vec![
                                TokenKind::keyword_if(),
                                TokenKind::List(Delimiter::Brace, vec![]),
                            ]),
                        )));
                    }
                }
            } else {
                return match self.step() {
                    Some(Token { kind, span }) => Some(Err(ParseError::tok(
                        kind.clone(),
                        *span,
                        ExpectedToken::SpecificTokens(vec![
                            TokenKind::keyword_else(),
                        ]),
                    ))),
                    None => Some(Err(ParseError::eoe(
                        if_span,
                        ExpectedToken::SpecificTokens(vec![
                            TokenKind::keyword_else(),
                        ]),
                    ))),
                };
            };

            Some(Ok(Expr {
                span: if_span,
                kind: ExprKind::Branch(Box::new(cond), Box::new(true_expr), Box::new(false_expr)),
            }))
        } else {
            None
        }
    }

    // If the inner content has comma(s), it interprets it as a tuple
    // Otherwise, it interprets it as a single expression inside a parenthesis
    // Some(Err(_)) indicates that it's a paren_expr but the inner expr has a syntax error
    pub fn step_paren_expr(&mut self) -> Option<Result<Expr, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::List(Delimiter::Parenthesis, elements),
                span,
            }) => {
                self.cursor += 1;

                let has_trailing_comma = match elements.last() {
                    Some(box Token { kind: TokenKind::Operator(OpToken::Comma), .. }) => true,
                    _ => false
                };

                let elements = match split_list_by_comma(elements) {
                    Ok(el) => el,
                    Err(er) => {
                        return Some(Err(er));
                    }
                };

                if elements.len() == 1 && !has_trailing_comma {
                    Some(Ok(Box::leak(elements[0].clone()).to_owned()))
                }

                else {
                    Some(Ok(Expr {
                        kind: ExprKind::Value(Value::tuple(elements, *span)),
                        span: *span,
                    }))
                }
            }
            _ => None,
        }
    }

    // works like `step_paren_expr` but for square brackets
    pub fn step_index_op(&mut self) -> Option<Result<Expr, ParseError>> {
        self._step_list_expr(Delimiter::Bracket)
    }

    // It works like `step_paren_expr`, but supports multiple args separated by commas
    // this function cannot distinguish between paren_expr and func_args -> the caller is responsible for that
    pub fn step_func_args(&mut self) -> Option<Result<Vec<Box<Expr>>, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::List(Delimiter::Parenthesis, elements),
                ..
            }) => {
                self.cursor += 1;

                Some(split_list_by_comma(elements))
            }
            _ => None,
        }
    }

    fn _step_list_expr(&mut self, delim: Delimiter) -> Option<Result<Expr, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::List(delim_, elements),
                ..
            }) if *delim_ == delim => {
                self.cursor += 1;
                let mut tokens = TokenList::from_vec_box_token(elements.clone());

                Some(parse_expr_exhaustive(&mut tokens))
            }
            _ => None,
        }
    }
}
