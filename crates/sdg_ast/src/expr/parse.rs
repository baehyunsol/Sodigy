use super::{Expr, ExprKind, InfixOp, PostfixOp, PrefixOp};
use crate::err::{ExpectedToken, ParseError};
use crate::span::Span;
use crate::token::{Token, TokenKind, TokenList};
use crate::value::parse_value;

// pratt algorithm
// https://github.com/matklad/minipratt
// https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
pub fn parse_expr(tokens: &mut TokenList, min_bp: u32) -> Result<Expr, ParseError> {
    let lhs_span = if let Some(span) = tokens.peek_curr_span() {
        span
    } else {
        // meaning there's no more token in the list
        return Err(ParseError::eoe(Span::dummy(), ExpectedToken::AnyExpression));
    };

    let mut lhs = if let Some(op) = tokens.step_prefix_op() {
        let bp = prefix_binding_power(op);
        let rhs = parse_expr(tokens, bp).map_err(|e| e.set_span_of_eof(lhs_span))?;

        Expr {
            span: lhs_span,
            kind: ExprKind::Prefix(op, Box::new(rhs)),
        }
    } else if let Some(expr) = tokens.step_paren_expr() {
        expr.map_err(|e| e.set_span_of_eof(lhs_span))?
    } else if let Some(branch) = tokens.step_branch_expr() {
        if let Ok(Expr { kind, .. }) = &branch {
            assert!(kind.is_branch(), "Internal Compiler Error 7DC70F8958E");
        }

        branch.map_err(|e| e.set_span_of_eof(lhs_span))?
    } else if let Some(match_expr) = tokens.step_match_expr() {
        if let Ok(Expr { kind, .. }) = &match_expr {
            assert!(kind.is_match(), "Internal Compiler Error C88E377CD8D");
        }

        match_expr.map_err(|e| e.set_span_of_eof(lhs_span))?
    } else {
        Expr {
            span: lhs_span,
            kind: ExprKind::Value(parse_value(tokens).map_err(|e| e.set_span_of_eof(lhs_span))?),
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
                    Box::new(index.map_err(|e| e.set_span_of_eof(curr_span))?),
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
                    args.map_err(|e| e.set_span_of_eof(curr_span))?,
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

            if op == InfixOp::Path && !tokens.peek_identifier().is_some() && !tokens.peek_number().is_some() {
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

            let rhs = parse_expr(tokens, r_bp).map_err(|e| e.set_span_of_eof(curr_span))?;

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
    }
}

fn prefix_binding_power(op: PrefixOp) -> u32 {
    match op {
        PrefixOp::Not | PrefixOp::Neg => NEG,
    }
}

// ref: https://doc.rust-lang.org/reference/expressions.html#expression-precedence
// ref: https://hexdocs.pm/elixir/main/operators.html
fn infix_binding_power(op: InfixOp) -> (u32, u32) {
    match op {
        InfixOp::Add | InfixOp::Sub => (ADD, ADD + 1),
        InfixOp::Mul | InfixOp::Div | InfixOp::Rem => (MUL, MUL + 1),
        InfixOp::Path => (PATH, PATH + 1),
        InfixOp::Index => (INDEX, INDEX + 1),
        InfixOp::Concat => (CONCAT, CONCAT + 1),
        InfixOp::Range => (RANGE, RANGE + 1),
        InfixOp::Gt | InfixOp::Lt | InfixOp::Ge | InfixOp::Le => (COMP, COMP + 1),
        InfixOp::Eq | InfixOp::Ne => (COMP_EQ, COMP_EQ + 1),
        InfixOp::BitwiseAnd => (BITWISE_AND, BITWISE_AND + 1),
        InfixOp::BitwiseOr => (BITWISE_OR, BITWISE_OR + 1),
        InfixOp::ModifyField(_) => (MODIFY, MODIFY + 1),
        InfixOp::LogicalAnd => (LOGICAL_AND, LOGICAL_AND + 1),
        InfixOp::LogicalOr => (LOGICAL_OR, LOGICAL_OR + 1),
    }
}

fn func_call_binding_power() -> (u32, u32) {
    (CALL, CALL + 1)
}

const PATH: u32 = 27;
const CALL: u32 = 25;
const INDEX: u32 = 23;
const NEG: u32 = 21;
const MUL: u32 = 19;
const ADD: u32 = 17;
const BITWISE_AND: u32 = 15;
const BITWISE_OR: u32 = 13;
const CONCAT: u32 = 11; const RANGE: u32 = 11;
const COMP: u32 = 9;
const COMP_EQ: u32 = 7;
const MODIFY: u32 = 5;
const LOGICAL_AND: u32 = 3;
const LOGICAL_OR: u32 = 1;
