use super::{Expr, ExprKind, InfixOp, NameOrigin, PostfixOp, PrefixOp};
use crate::err::{ExpectedToken, ParseError};
use crate::pattern::Pattern;
use crate::token::{Token, TokenKind, TokenList};
use crate::value::{parse_value, ValueKind};

/// pratt algorithm\
/// https://github.com/matklad/minipratt\
/// https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html\
pub fn parse_expr(tokens: &mut TokenList, min_bp: u32) -> Result<Expr, ParseError> {
    let lhs_span = if let Some(span) = tokens.peek_curr_span() {
        span
    } else {
        // meaning there's no more token in the list
        return Err(ParseError::eoe(tokens.get_eof_span(), ExpectedToken::AnyExpression));
    };

    let mut lhs = if let Some(op) = tokens.step_prefix_op() {
        let bp = prefix_binding_power(op);
        let rhs = parse_expr(tokens, bp)?;

        Expr {
            span: lhs_span,
            kind: ExprKind::Prefix(op, Box::new(rhs)),
        }
    } else if let Some(expr) = tokens.step_paren_expr() {
        expr?
    } else if let Some(branch) = tokens.step_branch_expr() {
        if let Ok(Expr { kind, .. }) = &branch {
            assert!(kind.is_branch(), "Internal Compiler Error 7DC70F8958E");
        }

        branch?
    } else if let Some(match_expr) = tokens.step_match_expr() {
        if let Ok(Expr { kind, .. }) = &match_expr {
            assert!(kind.is_match(), "Internal Compiler Error C88E377CD8D");
        }

        match_expr?
    } else {
        Expr {
            span: lhs_span,
            kind: ExprKind::Value(parse_value(tokens)?),
        }
    };

    loop {
        let curr_span = if let Some(span) = tokens.peek_curr_span() {
            span
        } else {
            break;
        };

        if let Some(op) = tokens.step_postfix_op() {
            let bp = postfix_binding_power(op);

            if bp < min_bp {
                tokens.backward(); // this operator is not parsed in this call
                break;
            }

            lhs = Expr {
                span: curr_span,
                kind: ExprKind::Postfix(op, Box::new(lhs)),
            };
            continue;
        }

        if let Some(index) = tokens.step_index_op() {
            let (l_bp, _) = infix_binding_power(InfixOp::Index);

            if l_bp < min_bp {
                tokens.backward(); // this operator is not parsed in this call
                break;
            }

            lhs = Expr {
                span: curr_span,
                kind: ExprKind::Infix(
                    InfixOp::Index,
                    Box::new(lhs),
                    Box::new(index?),
                ),
            };

            continue;
        }

        if let Some(args) = tokens.step_func_args() {
            let (l_bp, _) = func_call_binding_power();

            if l_bp < min_bp {
                tokens.backward(); // this operator is not parsed in this call
                break;
            }

            lhs = Expr {
                span: curr_span,
                kind: ExprKind::Call(
                    Box::new(lhs),
                    args?,
                ),
            };

            continue;
        }

        if let Some(op) = tokens.step_infix_op() {
            let op = op?;
            let (l_bp, r_bp) = infix_binding_power(op);

            if l_bp < min_bp {
                tokens.backward();  // this operator is not parsed in this call
                break;
            }

            let rhs = if op == InfixOp::Path {

                if !tokens.peek_identifier().is_some() && !tokens.peek_number().is_some() {
                    let err_msg = "A name of a field or a method must be an identifier or a number (for tuples).
    `a.b` is valid, but `a.(b)` is not.".to_string();
                    let expected = ExpectedToken::SpecificTokens(vec![
                        TokenKind::dummy_identifier(),
                        TokenKind::Number(1.into()),
                    ]);

                    if let Some(Token { kind, span }) = tokens.step() {
                        return Err(ParseError::tok_msg(
                            kind.clone(), *span,
                            expected, err_msg,
                        ));
                    }

                    else {
                        return Err(ParseError::eoe_msg(
                            curr_span,
                            expected,
                            err_msg,
                        ));
                    }
                }

                let mut rhs = parse_expr(tokens, r_bp)?;

                // it has to be done for `EnumDef::check_unused_generics`'s favor
                if let Expr { kind: ExprKind::Value(ValueKind::Identifier(_, origin)), .. } = &mut rhs {
                    *origin = NameOrigin::SubPath;
                }

                rhs
            } else {
                parse_expr(tokens, r_bp)?
            };

            lhs = Expr {
                span: curr_span,
                kind: ExprKind::Infix(op, Box::new(lhs), Box::new(rhs)),
            };

            continue;
        }

        break;
    }

    Ok(lhs)
}

fn postfix_binding_power(op: PostfixOp) -> u32 {
    match op {
        PostfixOp::Range => RANGE,
        PostfixOp::InclusiveRange => RANGE,
    }
}

fn prefix_binding_power(op: PrefixOp) -> u32 {
    match op {
        PrefixOp::Not | PrefixOp::Neg => NEG,
    }
}

/// ref: https://doc.rust-lang.org/reference/expressions.html#expression-precedence\
/// ref: https://hexdocs.pm/elixir/main/operators.html\
fn infix_binding_power(op: InfixOp) -> (u32, u32) {
    match op {
        InfixOp::Add | InfixOp::Sub => (ADD, ADD + 1),
        InfixOp::Mul | InfixOp::Div | InfixOp::Rem => (MUL, MUL + 1),
        InfixOp::Path => (PATH, PATH + 1),
        InfixOp::Index => (INDEX, INDEX + 1),
        InfixOp::Concat => (CONCAT, CONCAT + 1),
        InfixOp::Range | InfixOp::InclusiveRange => (RANGE, RANGE + 1),
        InfixOp::Gt | InfixOp::Lt | InfixOp::Ge | InfixOp::Le => (COMP, COMP + 1),
        InfixOp::Eq | InfixOp::Ne => (COMP_EQ, COMP_EQ + 1),
        InfixOp::BitwiseAnd => (BITWISE_AND, BITWISE_AND + 1),
        InfixOp::BitwiseOr => (BITWISE_OR, BITWISE_OR + 1),
        InfixOp::Append | InfixOp::Prepend => (APPEND, APPEND + 1),
        InfixOp::ModifyField(_) => (MODIFY, MODIFY + 1),
        InfixOp::LogicalAnd => (LOGICAL_AND, LOGICAL_AND + 1),
        InfixOp::LogicalOr => (LOGICAL_OR, LOGICAL_OR + 1),
    }
}

fn func_call_binding_power() -> (u32, u32) {
    (CALL, CALL + 1)
}

const PATH: u32 = 29;
const CALL: u32 = 27;
const INDEX: u32 = 25;
const NEG: u32 = 23;
const MUL: u32 = 21;
const ADD: u32 = 19;
const BITWISE_AND: u32 = 17;
const BITWISE_OR: u32 = 15;
const APPEND: u32 = 13;
const CONCAT: u32 = 11; const RANGE: u32 = 11;
const COMP: u32 = 9;
const COMP_EQ: u32 = 7;
const MODIFY: u32 = 5;
const LOGICAL_AND: u32 = 3;
const LOGICAL_OR: u32 = 1;

pub fn parse_match_body(tokens: &mut TokenList) -> Result<Vec<(Pattern, Expr)>, ParseError> {
    let mut branches = vec![];

    loop {
        let curr_pattern = match tokens.step_pattern() {
            Some(Ok(p)) => p,
            Some(Err(e)) => { return Err(e); },
            None => if branches.is_empty() {
                return Err(ParseError::eoe(
                    tokens.get_eof_span(),
                    ExpectedToken::AnyPattern,
                ));
            } else {
                return Ok(branches);
            },
        };

        tokens.consume_token_or_error(vec![TokenKind::right_arrow()])?;

        let curr_value = parse_expr(tokens, 0)?;

        branches.push((curr_pattern, curr_value));

        if tokens.consume(TokenKind::comma()) {
            continue;
        }
    }
}
