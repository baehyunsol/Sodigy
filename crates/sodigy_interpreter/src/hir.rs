use crate::{
    ConvertError,
    IntoHmath,
    SodigyData,
    SodigyDataType,
    SodigyDataValue,
    to_string,
    to_rust_string,
};
use sodigy_ast as ast;
use sodigy_high_ir as hir;
use sodigy_intern::{InternedString, unintern_string};
use sodigy_uid::Uid;
use std::collections::HashMap;
use std::rc::Rc;

pub struct HirEvalCtxt {
    func_arg_stack: Vec<Vec<Rc<SodigyData>>>,
    scoped_lets: Vec<HashMap<(Uid, InternedString), LazyEvalData>>,

    // it needs 2-step mapping because SodigyDataValue cannot store `Uid`s
    func_map: HashMap<Uid, usize>,
    funcs: HashMap<usize, hir::Func>,

    // it doesn't have to be 100% accurate
    pub call_depth: usize,
}

pub enum LazyEvalData {
    NotEvaled(hir::Expr),
    Evaled(Rc<SodigyData>),
}

impl HirEvalCtxt {
    pub fn from_session(sess: &hir::HirSession) -> Self {
        let mut func_map = HashMap::with_capacity(sess.func_defs.len());
        let mut funcs = HashMap::with_capacity(sess.func_defs.len());

        for func in sess.func_defs.values() {
            let index = func_map.len();

            func_map.insert(func.uid, index);
            funcs.insert(index, func.clone());
        }

        HirEvalCtxt {
            func_arg_stack: vec![],
            func_map,
            funcs,
            scoped_lets: vec![
                HashMap::new(),  // for main function
            ],
            call_depth: 0,
        }
    }

    pub fn push_scoped_let(&mut self, name: InternedString, uid: Uid, value: &hir::Expr) {
        let last_index = self.scoped_lets.len() - 1;
        self.scoped_lets[last_index].insert((uid, name), LazyEvalData::NotEvaled(value.clone()));
    }

    pub fn pop_scoped_let(&mut self, name: InternedString, uid: Uid) {
        let last_index = self.scoped_lets.len() - 1;
        self.scoped_lets[last_index].remove(&(uid, name)).unwrap();
    }

    pub fn get_scoped_let(&mut self, name: InternedString, uid: Uid) -> Option<&LazyEvalData> {
        let last_index = self.scoped_lets.len() - 1;
        self.scoped_lets[last_index].get(&(uid, name))
    }

    pub fn update_scoped_let(&mut self, name: InternedString, uid: Uid, value: Rc<SodigyData>) {
        let last_index = self.scoped_lets.len() - 1;

        match self.scoped_lets[last_index].get_mut(&(uid, name)) {
            Some(d) => {
                *d = LazyEvalData::Evaled(value);
            },
            _ => {},
        }
    }

    pub fn get_func_arg(&self, index: usize) -> Rc<SodigyData> {
        self.func_arg_stack.last().unwrap().get(index).unwrap().clone()
    }

    pub fn push_func_args(&mut self, args: Vec<Rc<SodigyData>>) {
        self.func_arg_stack.push(args);
        self.scoped_lets.push(HashMap::new());
    }

    pub fn pop_func_args(&mut self) {
        self.func_arg_stack.pop().unwrap();
        self.scoped_lets.pop().unwrap();
    }

    pub fn get_func_by_id(&self, index: usize) -> Option<&hir::Func> {
        self.funcs.get(&index)
    }

    pub fn get_func_by_uid(&self, index: Uid) -> Option<usize> {
        self.func_map.get(&index).map(|n| *n)
    }

    pub fn inc_call_depth(&mut self) {
        self.call_depth += 1;
    }

    pub fn dec_call_depth(&mut self) {
        self.call_depth -= 1;
    }
}

#[derive(Debug)]
pub enum HirEvalError {
    // not an error, but I haven't implemented this one yet
    TODO(String),

    // is an actual error, but I'm too lazy to declare a variant for that
    Msg(String),
}

pub fn eval_hir(e: &hir::Expr, ctxt: &mut HirEvalCtxt) -> Result<Rc<SodigyData>, HirEvalError> {
    if ctxt.call_depth > 256 {
        return Err(HirEvalError::Msg(String::from("call depth limit exceeded")));
    }

    match &e.kind {
        hir::ExprKind::Identifier(id_ori) => match id_ori.origin() {
                hir::NameOrigin::FuncArg { index } => Ok(ctxt.get_func_arg(*index)),
                hir::NameOrigin::Global { origin: Some(origin) } => {
                    let func_index = if let Some(index) = ctxt.get_func_by_uid(*origin) {
                        index
                    } else {
                        return Err(HirEvalError::TODO(format!("name resolving `{}`", id_ori.id().render_error())));
                    };

                    let func = if let Some(f) = ctxt.get_func_by_id(func_index) {
                        f
                    } else {
                        unreachable!()
                    };

                    if func.args.is_none() {
                        let func = func.clone();

                        ctxt.inc_call_depth();
                        let res = eval_hir(&func.return_val, ctxt);
                        ctxt.dec_call_depth();

                        res
                    }

                    else {
                        Ok(Rc::new(SodigyData::new_func(func_index)))
                    }
                },
                hir::NameOrigin::Local { origin } => match ctxt.get_scoped_let(id_ori.id(), *origin) {
                    Some(d) => match d {
                        LazyEvalData::NotEvaled(e) => {
                            let new_e = eval_hir(&e.clone(), ctxt)?;
                            ctxt.update_scoped_let(id_ori.id(), *origin, new_e.clone());

                            Ok(new_e)
                        },
                        LazyEvalData::Evaled(e) => Ok(e.clone()),
                    },
                    _ => Err(HirEvalError::TODO(String::from("Evaluating a scoped let... how can it fail?"))),
                },
                e => Err(HirEvalError::TODO(format!("ExprKind::Identifier other than FuncArg: {e:?}"))),
            },
        hir::ExprKind::Integer(n) => Ok(Rc::new(
            SodigyData::new_int(n.into_hmath_big_int()?)
        )),
        hir::ExprKind::Ratio(n) => {
            let n = n.into_hmath_ratio()?;
            let denom = n.get_denom();
            let numer = n.get_numer();

            Ok(Rc::new(
                SodigyData::new_ratio(denom, numer)
            ))
        },
        hir::ExprKind::Char(c) => Ok(Rc::new(
            SodigyData::new_char(*c)
        )),
        hir::ExprKind::String { s, is_binary } => if *is_binary {
            Ok(Rc::new(SodigyData::new_bin_data(&unintern_string(*s))))
        } else {
            Ok(Rc::new(SodigyData::new_string(&unintern_string(*s))))
        },
        hir::ExprKind::List(elements)
        | hir::ExprKind::Tuple(elements) => {
            let ty = if matches!(e.kind, hir::ExprKind::List(_)) { SodigyDataType::List } else { SodigyDataType::Tuple };
            let mut result = Vec::with_capacity(elements.len());
            ctxt.inc_call_depth();

            for elem in elements.iter() {
                result.push(eval_hir(elem, ctxt)?);
            }

            ctxt.dec_call_depth();
            Ok(Rc::new(SodigyData {
                value: SodigyDataValue::Compound(result),
                ty,
            }))
        },
        hir::ExprKind::Branch(hir::Branch { arms }) => {
            for hir::BranchArm { cond, value } in arms.iter() {
                if let Some(cond) = cond {
                    ctxt.inc_call_depth();
                    let cond = eval_hir(cond, ctxt)?;
                    ctxt.dec_call_depth();

                    if cond.is_true() {
                        ctxt.inc_call_depth();
                        let res = eval_hir(value, ctxt);
                        ctxt.dec_call_depth();

                        return res;
                    }

                    else {
                        continue;
                    }
                }

                else {
                    ctxt.inc_call_depth();
                    let res = eval_hir(value, ctxt);
                    ctxt.dec_call_depth();

                    return res;
                }
            }

            unreachable!()
        },
        hir::ExprKind::Format(elements) => {
            let mut result = Vec::with_capacity(elements.len());
            ctxt.inc_call_depth();

            for elem in elements.iter() {
                let e = eval_hir(elem, ctxt)?;
                let s = to_string(&e).map_err(
                    |_| HirEvalError::Msg(String::from("this type doesn't support `to_string`"))
                )?;
                let s = to_rust_string(&s).map_err(
                    |_| HirEvalError::Msg(String::from("this type doesn't support `to_rust_string`"))
                )?;

                result.push(s);
            }

            ctxt.dec_call_depth();
            Ok(Rc::new(SodigyData {
                value: SodigyDataValue::Compound(result.concat().iter().map(
                    |c| Rc::new(SodigyData {
                        value: SodigyDataValue::SmallInt(*c as i32),
                        ty: SodigyDataType::Char,
                    })
                ).collect()),
                ty: SodigyDataType::String,
            }))
        },
        hir::ExprKind::Call { func, args } => {
            let func = eval_hir(func, ctxt)?;
            let mut func_args = Vec::with_capacity(args.len());

            for arg in args.iter() {
                ctxt.inc_call_depth();
                func_args.push(eval_hir(arg, ctxt)?);
                ctxt.dec_call_depth();
            }

            let func_index = func.try_get_func_index().map_err(
                |_| HirEvalError::Msg(String::from("expected a function, got something else"))
            )?;
            let func = ctxt.get_func_by_id(func_index).ok_or_else(
                || HirEvalError::Msg(String::from("err with func index"))
            )?;

            match &func.args {
                None => {
                    return Err(HirEvalError::Msg(String::from("calling an uncallable function")));
                },
                Some(args) if args.len() != func_args.len() => {
                    return Err(HirEvalError::Msg(String::from("wrong number of args")));
                },
                _ => {},
            }

            let func = func.clone();

            ctxt.inc_call_depth();
            ctxt.push_func_args(func_args);
            let result = eval_hir(&func.return_val, ctxt);
            ctxt.pop_func_args();
            ctxt.dec_call_depth();

            result
        },
        hir::ExprKind::PrefixOp(op, val) => {
            ctxt.inc_call_depth();
            let val = eval_hir(val, ctxt)?;
            ctxt.dec_call_depth();

            match op {
                ast::PrefixOp::Neg => if let Some(n) = val.try_into_big_int() {
                    Ok(Rc::new(SodigyData::new_int(n.neg())))
                } else if let Some(n) = val.try_into_ratio() {
                    let res = n.neg();

                    Ok(Rc::new(SodigyData::new_ratio(res.get_denom(), res.get_numer())))
                } else {
                    Err(HirEvalError::TODO(String::from("negation")))
                },
                ast::PrefixOp::Not => Err(HirEvalError::TODO(String::from("logical not"))),
            }
        },
        hir::ExprKind::InfixOp(op, lhs, rhs) => {
            ctxt.inc_call_depth();
            let lhs = eval_hir(lhs, ctxt)?;
            let rhs = eval_hir(rhs, ctxt)?;
            ctxt.dec_call_depth();

            // let's not allow Int + Ratio and Ratio + Int. I want it to be explicit with types
            match op {
                ast::InfixOp::Add => if let (Some(m), Some(n)) = (
                    lhs.try_into_big_int(),
                    rhs.try_into_big_int(),
                ) {
                    Ok(Rc::new(SodigyData::new_int(m.add_bi(n))))
                } else if let (Some(m), Some(n)) = (
                    lhs.try_into_ratio(),
                    rhs.try_into_ratio(),
                ) {
                    let res = m.add_rat(&n);
                    Ok(Rc::new(SodigyData::new_ratio(res.get_denom(), res.get_numer())))
                } else {
                    Err(HirEvalError::TODO(String::from("addition")))
                },
                ast::InfixOp::Sub => if let (Some(m), Some(n)) = (
                    lhs.try_into_big_int(),
                    rhs.try_into_big_int(),
                ) {
                    Ok(Rc::new(SodigyData::new_int(m.sub_bi(n))))
                } else if let (Some(m), Some(n)) = (
                    lhs.try_into_ratio(),
                    rhs.try_into_ratio(),
                ) {
                    let res = m.sub_rat(&n);
                    Ok(Rc::new(SodigyData::new_ratio(res.get_denom(), res.get_numer())))
                } else {
                    Err(HirEvalError::TODO(String::from("subtraction")))
                },
                ast::InfixOp::Mul => if let (Some(m), Some(n)) = (
                    lhs.try_into_big_int(),
                    rhs.try_into_big_int(),
                ) {
                    Ok(Rc::new(SodigyData::new_int(m.mul_bi(n))))
                } else if let (Some(m), Some(n)) = (
                    lhs.try_into_ratio(),
                    rhs.try_into_ratio(),
                ) {
                    let res = m.mul_rat(&n);
                    Ok(Rc::new(SodigyData::new_ratio(res.get_denom(), res.get_numer())))
                } else {
                    Err(HirEvalError::TODO(String::from("multiplication")))
                },
                ast::InfixOp::Div => if let (Some(m), Some(n)) = (
                    lhs.try_into_big_int(),
                    rhs.try_into_big_int(),
                ) {
                    Ok(Rc::new(SodigyData::new_int(m.div_bi(n))))
                } else if let (Some(m), Some(n)) = (
                    lhs.try_into_ratio(),
                    rhs.try_into_ratio(),
                ) {
                    let res = m.div_rat(&n);
                    Ok(Rc::new(SodigyData::new_ratio(res.get_denom(), res.get_numer())))
                } else {
                    Err(HirEvalError::TODO(String::from("division")))
                },
                ast::InfixOp::Rem => Err(HirEvalError::TODO(String::from("remainder"))),
                ast::InfixOp::Eq => if let (Some(m), Some(n)) = (
                    lhs.try_into_big_int(),
                    rhs.try_into_big_int(),
                ) {
                    Ok(Rc::new(m.eq_bi(n).into()))
                } else if let (Some(m), Some(n)) = (
                    lhs.try_into_ratio(),
                    rhs.try_into_ratio(),
                ) {
                    Ok(Rc::new(m.eq_rat(&n).into()))
                } else {
                    Err(HirEvalError::TODO(String::from("eq")))
                },
                ast::InfixOp::Gt => if let (Some(m), Some(n)) = (
                    lhs.try_into_big_int(),
                    rhs.try_into_big_int(),
                ) {
                    Ok(Rc::new(m.gt_bi(n).into()))
                } else if let (Some(m), Some(n)) = (
                    lhs.try_into_ratio(),
                    rhs.try_into_ratio(),
                ) {
                    Ok(Rc::new(m.gt_rat(&n).into()))
                } else {
                    Err(HirEvalError::TODO(String::from("gt")))
                },
                ast::InfixOp::Lt => if let (Some(m), Some(n)) = (
                    lhs.try_into_big_int(),
                    rhs.try_into_big_int(),
                ) {
                    Ok(Rc::new(m.lt_bi(n).into()))
                } else if let (Some(m), Some(n)) = (
                    lhs.try_into_ratio(),
                    rhs.try_into_ratio(),
                ) {
                    Ok(Rc::new(m.lt_rat(&n).into()))
                } else {
                    Err(HirEvalError::TODO(String::from("lt")))
                },
                // TODO: it's not lazily evaluated
                ast::InfixOp::LogicalOr => Ok(Rc::new((lhs.is_true() || rhs.is_true()).into())),
                // TODO: it's not lazily evaluated
                ast::InfixOp::LogicalAnd => Ok(Rc::new((lhs.is_true() && rhs.is_true()).into())),
                ast::InfixOp::Index => {
                    if let (
                        SodigyData {
                            value: SodigyDataValue::Compound(elems),
                            ty: SodigyDataType::List,
                        },
                        SodigyData {
                            value: SodigyDataValue::BigInt(n),
                            ty: SodigyDataType::Integer,
                        },
                    ) = (lhs.as_ref(), rhs.as_ref()) {
                        if let Ok(n) = i64::try_from(n) {
                            if n < 0 {
                                Err(HirEvalError::TODO(String::from("negative index")))
                            }

                            else {
                                if (n as usize) < elems.len() {
                                    Ok(elems[n as usize].clone())
                                }

                                else {
                                    Err(HirEvalError::Msg(String::from("invalid index: too big")))
                                }
                            }
                        }

                        else {
                            Err(HirEvalError::Msg(String::from("invalid index")))
                        }
                    }

                    else {
                        Err(HirEvalError::Msg(String::from("ty error in an index operation (for now, it only supports list[int])")))
                    }
                },
                // TODO: check types
                ast::InfixOp::Concat => match (
                    lhs.as_ref(), rhs.as_ref()
                ) {
                    (
                        SodigyData {
                            value: SodigyDataValue::Compound(l1),
                            ty: SodigyDataType::List,
                        },
                        SodigyData {
                            value: SodigyDataValue::Compound(l2),
                            ty: SodigyDataType::List,
                        },
                    ) => {
                        Ok(Rc::new(SodigyData {
                            value: SodigyDataValue::Compound(vec![l1.clone(), l2.clone()].concat()),
                            ty: SodigyDataType::List,
                        }))
                    },
                    _ => Err(HirEvalError::TODO(String::from("`<>` for other types")))
                },
                _ => Err(HirEvalError::TODO(op.to_string())),
            }
        },
        hir::ExprKind::Scope(hir::expr::Scope {
            lets, value, uid, ..
        }) => {
            for hir::expr::ScopedLet { name, value, .. } in lets.iter() {
                ctxt.push_scoped_let(name.id(), *uid, value);
            }

            let res = eval_hir(value, ctxt);

            for hir::expr::ScopedLet { name, .. } in lets.iter() {
                ctxt.pop_scoped_let(name.id(), *uid);
            }

            res
        },
        _ => Err(HirEvalError::TODO(format!("not implemented yet: {e}"))),
    }
}

impl From<ConvertError> for HirEvalError {
    fn from(e: ConvertError) -> HirEvalError {
        match e {
            ConvertError::TODO(s) => HirEvalError::TODO(s),
            ConvertError::NotInt => HirEvalError::Msg(String::from("ConvertError::NotInt")),
            ConvertError::NotRatio => HirEvalError::Msg(String::from("ConvertError::NotRatio")),
        }
    }
}
