// Very experimental mir evaluation

use sodigy_error::Error;
use sodigy_mir::{self as mir, Callable, Expr};
use sodigy_name_analysis::NameOrigin;
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::intern_string;
use sodigy_token::InfixOp;
use std::collections::HashMap;

mod stack;
mod value;

pub use stack::Stack;
pub use value::Value;

pub fn eval_main(
    session: &mir::Session,
) -> Result<Value, Error> {
    let funcs = session.funcs.iter().map(
        |func| (
            func.name_span,
            func.clone(),
        )
    ).collect::<HashMap<_, _>>();

    // Ideal way: draw dependency graph between `let` values and initialize the values according to the dependency (+ cycle detection)
    // Stupid way (current impl): just try eval the value a few times and wish everything is fine.
    let mut lets = HashMap::new();

    for _ in 0..3 {
        for r#let in session.lets.iter() {
            let mut stack = Stack::new();

            if let Ok(value) = eval(&r#let.value, &funcs, &lets, &mut stack) {
                lets.insert(r#let.name_span, value);
            }
        }
    }

    for func in session.funcs.iter() {
        if func.name == intern_string(b"main") {
            let mut stack = Stack::new();
            return Ok(eval(
                &func.value,
                &funcs,
                &lets,
                &mut stack,
            )?);
        }
    }

    panic!("No main function...");
}

fn eval(
    expr: &Expr,
    funcs: &HashMap<Span, mir::Func>,
    lets: &HashMap<Span, Value>,
    stack: &mut Stack,
) -> Result<Value, Error> {
    match expr {
        Expr::Identifier(id) => match id.origin {
            NameOrigin::FuncArg { index, .. } => Ok(stack.func_args.last().unwrap()[index].clone()),
            NameOrigin::Local { .. } |
            NameOrigin::Foreign { .. } => match lets.get(&id.def_span) {
                Some(value) => Ok(value.clone()),

                // Some functors (and all lambda functions) are
                // in `funcs`. So we just have to turn this into a dynamic functor.
                None => match funcs.get(&id.def_span) {
                    Some(_) => Ok(Value::Functor(id.def_span)),
                    None => todo!(),
                },
            },
        },
        Expr::Number { n, .. } => Ok(Value::Number(*n)),
        Expr::Block(block) => {
            for r#let in block.lets.iter() {
                todo!()
            }

            Ok(eval(
                &block.value,
                funcs,
                lets,
                stack,
            )?)
        },
        Expr::If(r#if) => {
            let cond = eval(&r#if.cond, funcs, lets, stack)?;

            match cond {
                Value::Bool(true) => Ok(eval(
                    &r#if.true_value,
                    funcs,
                    lets,
                    stack,
                )?),
                Value::Bool(false) => Ok(eval(
                    &r#if.false_value,
                    funcs,
                    lets,
                    stack,
                )?),
                _ => todo!(),
            }
        },
        Expr::Call { func, args } => {
            let mut arg_values = Vec::with_capacity(args.len());

            for arg in args.iter() {
                arg_values.push(eval(arg, funcs, lets, stack)?);
            }

            match func {
                Callable::Static { def_span, .. } => match funcs.get(def_span) {
                    Some(func) => {
                        stack.func_args.push(arg_values);
                        let r = eval(&func.value, funcs, lets, stack)?;
                        stack.func_args.pop();
                        Ok(r)
                    },
                    _ => todo!(),
                },
                Callable::Dynamic(expr) => {
                    let functor = eval(expr, funcs, lets, stack)?;

                    match functor {
                        Value::Functor(def_span) => match funcs.get(&def_span) {
                            Some(func) => {
                                stack.func_args.push(arg_values);
                                let r = eval(&func.value, funcs, lets, stack)?;
                                stack.func_args.pop();
                                Ok(r)
                            },
                            None => todo!(),
                        },
                        _ => todo!(),
                    }
                },
                Callable::ListInit { .. } => Ok(Value::List(arg_values)),
                Callable::GenericInfixOp { op, .. } => {
                    let (lhs, rhs) = (&arg_values[0], &arg_values[1]);

                    match op {
                        InfixOp::Add => match (lhs, rhs) {
                            (Value::Number(n), Value::Number(m)) => match (n, m) {
                                (InternedNumber::SmallInteger(n), InternedNumber::SmallInteger(m)) => Ok(Value::Number(InternedNumber::SmallInteger(*n + *m))),
                                _ => todo!(),
                            },
                            _ => todo!(),
                        },
                        InfixOp::Sub => match (lhs, rhs) {
                            (Value::Number(n), Value::Number(m)) => match (n, m) {
                                (InternedNumber::SmallInteger(n), InternedNumber::SmallInteger(m)) => Ok(Value::Number(InternedNumber::SmallInteger(*n - *m))),
                                _ => todo!(),
                            },
                            _ => todo!(),
                        },
                        InfixOp::Mul => match (lhs, rhs) {
                            (Value::Number(n), Value::Number(m)) => match (n, m) {
                                (InternedNumber::SmallInteger(n), InternedNumber::SmallInteger(m)) => Ok(Value::Number(InternedNumber::SmallInteger(*n * *m))),
                                _ => todo!(),
                            },
                            _ => todo!(),
                        },
                        InfixOp::Lt => match (lhs, rhs) {
                            (Value::Number(n), Value::Number(m)) => match (n, m) {
                                (InternedNumber::SmallInteger(n), InternedNumber::SmallInteger(m)) => Ok(Value::Bool(*n < *m)),
                                _ => todo!(),
                            },
                            _ => todo!(),
                        },
                        InfixOp::Eq => match (lhs, rhs) {
                            (Value::Number(n), Value::Number(m)) => match (n, m) {
                                (InternedNumber::SmallInteger(n), InternedNumber::SmallInteger(m)) => Ok(Value::Bool(*n == *m)),
                                _ => todo!(),
                            },
                            _ => todo!(),
                        },
                        _ => panic!("TODO: {op:?}"),
                    }
                },
                _ => panic!("TODO: {func:?}"),
            }
        },
        _ => panic!("TODO: {expr:?}"),
    }
}
