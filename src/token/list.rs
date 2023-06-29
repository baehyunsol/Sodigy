use super::{Delimiter, Keyword, Token, TokenKind, OpToken};
use crate::err::ParseError;
use crate::expr::{Expr, ExprKind, parse_expr, PrefixOp, InfixOp, PostfixOp};
use crate::parse::split_list_by_comma;
use crate::span::Span;

pub struct TokenList {
    pub data: Vec<Token>,
    cursor: usize
}

impl TokenList {

    pub fn from_vec(data: Vec<Token>) -> Self {
        TokenList {
            data, cursor: 0
        }
    }

    pub fn from_vec_box_token(data: Vec<Box<Token>>) -> Self {
        TokenList {
            data: data.into_iter().map(|token| Box::leak(token).to_owned()).collect(),
            cursor: 0
        }
    }

    pub fn is_eof(&self) -> bool {
        #[cfg(test)] assert!(self.cursor <= self.data.len());

        self.cursor >= self.data.len()
    }

    pub fn backward(&mut self) {
        self.cursor -= 1;
    }

    // if the current token is `token`, it steps forward and returns true
    // it returns false otherwise
    // it's helpful if the borrow checker doesn't allow you to use `self.step`
    pub fn consume(&mut self, token: TokenKind) -> bool {

        match self.data.get(self.cursor) {
            Some(t) if t.kind == token => {
                self.cursor += 1;
                true
            },
            _ => false
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

    pub fn step(&mut self) -> Option<&Token> {
        let result = self.data.get(self.cursor);

        if result.is_some() {
            self.cursor += 1;
        }

        result
    }

    pub fn step_prefix_op(&mut self) -> Option<PrefixOp> {

        match self.data.get(self.cursor) {
            Some(Token { kind: TokenKind::Operator(op), .. }) => match op {
                OpToken::Sub | OpToken::Not => {
                    self.cursor += 1;
                    Some(op.into())
                },
                _ => None
            },
            _ => None
        }

    }

    pub fn step_infix_op(&mut self) -> Option<InfixOp> {

        match self.data.get(self.cursor) {
            Some(Token { kind: TokenKind::Operator(op), .. }) => match op {
                OpToken::Add | OpToken::Sub | OpToken::Mul | OpToken::Div | OpToken::Rem
                | OpToken::Eq | OpToken::Gt | OpToken::Lt | OpToken::Ne | OpToken::Ge | OpToken::Le
                | OpToken::Dot | OpToken::DotDot | OpToken::Concat
                | OpToken::And | OpToken::AndAnd | OpToken::Or | OpToken::OrOr  => {
                    self.cursor += 1;
                    Some(op.into())
                },
                _ => None
            },
            _ => None
        }

    }

    pub fn step_postfix_op(&mut self) -> Option<PostfixOp> {

        match self.data.get(self.cursor) {
            Some(Token { kind: TokenKind::Operator(op), .. }) => match op {

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
                            },

                            // infix
                            OpToken::Sub | OpToken::Not => None,

                            // TODO: not sure what to do
                            _ => {
                                self.cursor += 1;
                                Some(op.into())
                            }
                        },

                        // infix
                        _ => None
                    }

                },
                _ => None
            },
            _ => None
        }

    }

    // Some(Err(_)) indicates that it's a branch_expr but the inner expr has a syntax error
    // if self.step() is `if`, it must return Some(_)
    pub fn step_branch_expr(&mut self) -> Option<Result<Expr, ParseError>> {
        let if_span = if let Some(s) = self.get_curr_span() { s } else { return None; };

        if self.consume(TokenKind::Keyword(Keyword::If)) {

            let cond = match parse_expr(self, 0) {
                Ok(expr) => expr,
                Err(e) => { return Some(Err(e.set_span_of_eof(if_span))); }
            };

            let true_expr = match self.step() {
                Some(Token { kind: TokenKind::List(Delimiter::Brace, elements), span: true_expr_span }) => {
                    let mut true_expr_tokens = TokenList::from_vec_box_token(elements.to_vec());

                    match parse_expr(&mut true_expr_tokens, 0) {
                        Ok(expr) if true_expr_tokens.is_eof() => expr,
                        Ok(_) => {  // has unnecessary tokens
                            let err_token = true_expr_tokens.step().unwrap();

                            return Some(Err(ParseError::tok(err_token.kind.clone(), err_token.span)));
                        },
                        Err(e) => {
                            return Some(Err(e.set_span_of_eof(*true_expr_span)));
                        }
                    }
                },
                Some(Token { kind, span }) => {
                    return Some(Err(ParseError::tok(kind.clone(), span.clone())));
                }
                None => {
                    return Some(Err(ParseError::eoe(if_span)));
                }
            };
            let else_span = self.get_curr_span();
            let false_expr = if self.consume(TokenKind::Keyword(Keyword::Else)) {

                match self.data.get(self.cursor) {
                    Some(Token { kind: TokenKind::Keyword(Keyword::If), .. }) => {

                        match self.step_branch_expr() {
                            Some(Ok(false_expr)) => false_expr,
                            Some(Err(e)) => {
                                return Some(Err(e));
                            }
                            None => unreachable!()
                        }

                    },
                    Some(Token { kind: TokenKind::List(Delimiter::Brace, elements), span: false_expr_span }) => {
                        self.cursor += 1;
                        let mut false_expr_tokens = TokenList::from_vec_box_token(elements.to_vec());

                        match parse_expr(&mut false_expr_tokens, 0) {
                            Ok(expr) if false_expr_tokens.is_eof() => expr,
                            Ok(_) => {  // has unnecessary tokens
                                let err_token = false_expr_tokens.step().unwrap();

                                return Some(Err(ParseError::tok(err_token.kind.clone(), err_token.span)));
                            },
                            Err(e) => {
                                return Some(Err(e.set_span_of_eof(*false_expr_span)));
                            }
                        }
                    },
                    Some(Token { kind, span }) => {
                        return Some(Err(ParseError::tok(kind.clone(), span.clone())))
                    },
                    None => {
                        return Some(Err(ParseError::eoe(else_span.unwrap())));
                    }
                }

            }

            // if an `if_expr` doesn't have an else branch, it inserts `unreachable!`
            else {
                todo!()
            };

            Some(Ok(Expr { span: if_span, kind: ExprKind::Branch(Box::new(cond), Box::new(true_expr), Box::new(false_expr)) }))
        }

        else {
            None
        }

    }

    // SINGLE expr inside a parenthesis
    // Some(Err(_)) indicates that it's a paren_expr but the inner expr has a syntax error
    pub fn step_paren_expr(&mut self) -> Option<Result<Expr, ParseError>> {
        self._step_list_expr(Delimiter::Parenthesis)
    }

    // works like `step_paren_expr` but for square brackets
    pub fn step_index_op(&mut self) -> Option<Result<Expr, ParseError>> {
        self._step_list_expr(Delimiter::Bracket)
    }

    // It works like `step_paren_expr`, but supports multiple args separated by commas
    // this function cannot distinguish between paren_expr and func_args -> the caller is responsible for that
    pub fn step_func_args(&mut self) -> Option<Result<Vec<Box<Expr>>, ParseError>> {

        match self.data.get(self.cursor) {
            Some(Token { kind: TokenKind::List(Delimiter::Parenthesis, elements), .. }) => {
                self.cursor += 1;

                Some(split_list_by_comma(elements))
            },
            _ => None
        }

    }

    fn _step_list_expr(&mut self, delim: Delimiter) -> Option<Result<Expr, ParseError>> {
        match self.data.get(self.cursor) {
            Some(Token { kind: TokenKind::List(delim_, elements), .. }) if *delim_ == delim => {
                self.cursor += 1;

                Some(parse_expr(
                    &mut TokenList::from_vec_box_token(elements.clone()), 0
                ))
            },
            _ => None
        }
    }
}