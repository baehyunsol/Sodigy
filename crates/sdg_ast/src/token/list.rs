use super::{Delimiter, Keyword, OpToken, Token, TokenKind};
use crate::err::{ExpectedToken, ParseError};
use crate::expr::{parse_match_body, parse_expr, Expr, ExprKind, InfixOp, PostfixOp, PrefixOp};
use crate::parse::{parse_expr_exhaustive, split_list_by_comma};
use crate::pattern::Pattern;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::stmt::{parse_arg_def, ArgDef, GenericDef};
use crate::value::{parse_block_expr, ValueKind};
use hmath::Ratio;

pub struct TokenList {
    pub data: Vec<Token>,
    cursor: usize,
    first_character: Span,
}

impl TokenList {
    pub fn from_vec(data: Vec<Token>, first_character: Span) -> Self {
        TokenList { data, cursor: 0, first_character }
    }

    pub fn is_eof(&self) -> bool {
        assert!(
            self.cursor <= self.data.len(),
            "Internal Compiler Error 2E10FE0E985"
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

    // when a parser throws an Eof or Eoe error, this function is used to calc the span
    pub fn get_eof_span(&self) -> Span {
        match self.data.last() {
            Some(t) => t.span.last_character(),
            _ => self.first_character,
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

    pub fn march_until_stmt_begin(&mut self) {
        while let Some(token) = self.data.get(self.cursor) {
            if token.kind.is_stmt_begin() {
                return;
            }

            self.cursor += 1;
        }
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

    // it only consumes one token
    // the input is `Vec<TokenKind>` because it may expect multiple KINDs of A token
    pub fn consume_token_or_error(&mut self, token_kinds: Vec<TokenKind>) -> Result<(), ParseError> {
        match self.step() {
            Some(Token { kind, .. }) if token_kinds.contains(kind) => Ok(()),
            Some(Token { kind, span }) => Err(ParseError::tok(
                kind.clone(),
                *span,
                ExpectedToken::SpecificTokens(token_kinds),
            )),
            None => Err(ParseError::eoe(
                self.get_eof_span(),
                ExpectedToken::SpecificTokens(token_kinds),
            )),
        }
    }

    // `peek_XXX` functions don't move the cursor
    // it returns None if the cursor is not pointing to the data

    pub fn peek_curr_span(&self) -> Option<Span> {
        self.data.get(self.cursor).map(|t| t.span)
    }

    pub fn peek_identifier(&self) -> Option<InternedString> {
        match self.data.get(self.cursor) {
            Some(t) if t.is_identifier() => Some(t.unwrap_identifier()),
            _ => None
        }
    }

    pub fn peek_number(&self) -> Option<Ratio> {
        match self.data.get(self.cursor) {
            Some(t) if t.is_number() => Some(t.unwrap_number()),
            _ => None
        }
    }

    // `step_XXX` functions (including `step`)
    // if the current token is `XXX`, it returns `Some(XXX)` and steps the cursor forward
    // otherwise, it doesn't do anything and returns `None`
    // `step_XXX_strict` are like `step_XXX`, but returns `Err()` instead of `None`

    pub fn step(&mut self) -> Option<&Token> {
        let result = self.data.get(self.cursor);

        if result.is_some() {
            self.cursor += 1;
        }

        result
    }

    // it turns `TokenKind::List(_, elements)` into `TokenList::from_vec(elements)`
    pub fn step_grouped_tokens_strict(&mut self, delim: Delimiter, eof_span: Span) -> Result<TokenList, ParseError> {
        match self.step() {
            Some(Token {
                kind: TokenKind::List(delim_, elements),
                span: list_span,
            }) if *delim_ == delim => Ok(TokenList::from_vec(elements.to_vec(), list_span.first_character())),
            Some(Token { kind, span }) => {
                Err(ParseError::tok(
                    kind.clone(),
                    *span,
                    ExpectedToken::SpecificTokens(vec![
                        delim.opening_token_kind(),
                    ]),
                ))
            }
            None => {
                Err(ParseError::eoe(
                    eof_span,
                    ExpectedToken::SpecificTokens(vec![
                        delim.opening_token_kind(),
                    ]),
                ))
            }
        }
    }

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
                self.get_eof_span(),
                ExpectedToken::SpecificTokens(vec![
                    TokenKind::dummy_identifier(),
                ]),
            ))
        }
    }

    pub fn step_generic_defs(&mut self) -> Option<Result<Vec<GenericDef>, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::Operator(OpToken::Lt),
                span: generic_def_span,
            }) => {
                self.cursor += 1;
                let mut generics = vec![];

                // no borrowck
                let generic_def_span = *generic_def_span;

                loop {
                    if self.consume(TokenKind::Operator(OpToken::Gt)) {
                        break;
                    }

                    let name_span = self.peek_curr_span();
                    let name = match self.step_identifier_strict() {
                        Ok(n) => n,
                        Err(e) => {
                            return Some(Err(e));
                        }
                    };
                    let name_span = name_span.expect("Internal Compiler Error 0AA5B43F2C1");
                    generics.push(GenericDef::new(name, name_span));

                    if self.consume(TokenKind::comma()) {
                        continue;
                    } else if self.consume(TokenKind::Operator(OpToken::Gt)) {
                        break;
                    } else {
                        // must be an error
                        if let Err(e) = self.consume_token_or_error(vec![
                            TokenKind::comma(),
                            TokenKind::Operator(OpToken::Gt),
                        ]) {
                            return Some(Err(e));
                        }
                    }
                }

                Some(Ok(generics))
            },
            _ => None,
        }
    }

    pub fn step_func_def_args(&mut self) -> Option<Result<Vec<ArgDef>, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::List(Delimiter::Parenthesis, elements),
                span,
            }) => {
                self.cursor += 1;
                let mut args = vec![];
                let mut args_tokens = TokenList::from_vec(elements.to_vec(), span.first_character());

                while !args_tokens.is_eof() {
                    match parse_arg_def(&mut args_tokens) {
                        Ok(arg) => {
                            args.push(arg);
                        }
                        Err(e) => {
                            return Some(Err(e));
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

    pub fn step_pattern(&mut self) -> Option<Result<Pattern, ParseError>> {
        todo!()
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

    // There's one case where it returns an error: an identifier doesn't follow a `$`.
    pub fn step_infix_op(&mut self) -> Option<Result<InfixOp, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::Operator(op),
                span,
            }) => match *op {
                OpToken::Dollar => {
                    self.cursor += 1;
                    let span = *span;
                    let field_name = match self.step_identifier_strict() {
                        Ok(f) => f,
                        Err(mut e) => {
                            if e.is_unexpected_token() {
                                e.set_msg("A name of a field must follow `$`.");
                            }

                            return Some(Err(e));
                        }
                    };

                    Some(Ok(InfixOp::ModifyField(field_name)))
                },
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
                    Some(Ok(op.into()))
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

    // Some(Err(_)) indicates that it's a match_expr but the inner expr has a syntax error
    // if self.step() is `match`, it must return Some(_)
    pub fn step_match_expr(&mut self) -> Option<Result<Expr, ParseError>> {
        let match_span = if let Some(s) = self.peek_curr_span() {
            s
        } else {
            return None;
        };

        if self.consume(TokenKind::keyword_match()) {
            let value = match parse_expr(self, 0) {
                Ok(expr) => expr,
                Err(e) => {
                    return Some(Err(e));
                }
            };

            match self.step_grouped_tokens_strict(Delimiter::Brace, match_span) {
                Ok(mut tokens) => match parse_match_body(&mut tokens) {
                    Ok(_) => todo!(),
                    Err(e) => Some(Err(e)),
                },
                Err(e) => Some(Err(e)),
            }
        }

        else {
            None
        }
    }

    // Some(Err(_)) indicates that it's a branch_expr but the inner expr has a syntax error
    // if self.step() is `if`, it must return Some(_)
    pub fn step_branch_expr(&mut self) -> Option<Result<Expr, ParseError>> {
        let if_span = if let Some(s) = self.peek_curr_span() {
            s
        } else {
            return None;
        };

        if self.consume(TokenKind::keyword_if()) {
            let cond = match parse_expr(self, 0) {
                Ok(expr) => expr,
                Err(e) => {
                    return Some(Err(e));
                }
            };

            let true_expr = match self.step_grouped_tokens_strict(Delimiter::Brace, if_span) {
                Ok(mut tokens) => {
                    let true_expr_span = tokens.peek_curr_span().unwrap_or(if_span);

                    match parse_block_expr(&mut tokens) {
                        Ok(t) => t.try_unwrap_block_value(true_expr_span),
                        Err(e) => {
                            return Some(Err(e));
                        }
                    }
                },
                Err(e) => {
                    return Some(Err(e));
                }
            };

            let else_span = self.peek_curr_span();

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
                            None => unreachable!("Internal Compiler Error 438E6E8F21A"),
                        }
                    },
                    Some(Token {
                        kind: TokenKind::List(Delimiter::Brace, elements),
                        span: false_expr_span,
                    }) => match parse_block_expr(
                        &mut TokenList::from_vec(elements.to_vec(), false_expr_span.first_character())
                    ) {
                        Ok(t) => t.try_unwrap_block_value(*false_expr_span),
                        Err(e) => {
                            return Some(Err(e));
                        }
                    }
                    Some(Token { kind, span }) => {
                        return Some(Err(ParseError::tok(
                            kind.clone(),
                            *span,
                            ExpectedToken::SpecificTokens(vec![
                                TokenKind::keyword_if(),
                                TokenKind::opening_curly_brace(),
                            ]),
                        )))
                    }
                    None => {
                        return Some(Err(ParseError::eoe(
                            else_span.expect("Internal Compiler Error A862F21BFAD"),
                            ExpectedToken::SpecificTokens(vec![
                                TokenKind::keyword_if(),
                                TokenKind::opening_curly_brace(),
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
                        self.get_eof_span(),
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
                    Some(Token { kind: TokenKind::Operator(OpToken::Comma), .. }) => true,
                    _ => false
                };

                let elements = match split_list_by_comma(elements) {
                    Ok(el) => el,
                    Err(mut er) => {
                        er.set_expected_tokens_instead_of_nothing(vec![
                            TokenKind::Operator(OpToken::ClosingParenthesis),
                            TokenKind::comma(),
                        ]);
                        return Some(Err(er));
                    }
                };

                if elements.len() == 1 && !has_trailing_comma {
                    Some(Ok(elements[0].clone()))
                }

                else {
                    Some(Ok(Expr {
                        kind: ExprKind::Value(ValueKind::Tuple(elements)),
                        span: *span,
                    }))
                }
            }
            _ => None,
        }
    }

    // works like `step_paren_expr` but for square brackets
    pub fn step_index_op(&mut self) -> Option<Result<Expr, ParseError>> {
        self._step_list_expr(Delimiter::Bracket).map(
            |r| r.map_err(|mut e| {
                e.set_expected_tokens_instead_of_nothing(vec![
                    TokenKind::Operator(OpToken::ClosingSquareBracket),
                ]);

                e
            })
        )
    }

    // It works like `step_paren_expr`, but supports multiple args separated by commas
    // this function cannot distinguish between paren_expr and func_args -> the caller is responsible for that
    pub fn step_func_args(&mut self) -> Option<Result<Vec<Expr>, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::List(Delimiter::Parenthesis, elements),
                ..
            }) => {
                self.cursor += 1;

                Some(
                    split_list_by_comma(elements).map_err(
                        |mut e| {
                            e.set_expected_tokens_instead_of_nothing(vec![
                                TokenKind::Operator(OpToken::ClosingParenthesis),
                                TokenKind::comma(),
                            ]);

                            e
                        }
                    )
                )
            }
            _ => None,
        }
    }

    fn _step_list_expr(&mut self, delim: Delimiter) -> Option<Result<Expr, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token {
                kind: TokenKind::List(delim_, elements),
                span,
            }) if *delim_ == delim => {
                self.cursor += 1;
                let mut tokens = TokenList::from_vec(elements.clone(), span.first_character());

                Some(parse_expr_exhaustive(&mut tokens))
            }
            _ => None,
        }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        format!(
            "({}, [{}])",
            self.cursor,
            self.data.iter().map(
                |t| t.dump(session)
            ).collect::<Vec<String>>().join(", ")
        )
    }
}
