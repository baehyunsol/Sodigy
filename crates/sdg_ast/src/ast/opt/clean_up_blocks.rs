use super::super::{AST, ASTError, NameOrigin};
use super::substitute_local_def;
use crate::err::ParamType;
use crate::expr::{Expr, ExprKind, MatchBranch};
use crate::iter_mut_exprs_in_ast;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::stmt::{ArgDef, Decorator};
use crate::value::{BlockDef, ValueKind};
use crate::warning::SodigyWarning;
use sdg_uid::UID;
use std::collections::{HashMap, HashSet};

/*
TODO

{
    x = foo();

    if cond {
        f(x)
    } else {
        g(x)
    }
}

even though `x` is used twice (syntactically), it'll never be used twice (semantically)
*/

/// 1. If a definition is used only once, the value goes directly to the used place.
/// 2. If a definition is used 0 times, it's removed.
/// 3. If a value of a definition is simple, all the referents are replaced with the value.
///   - simple value: single identifier (or a path), small number (how small?), static values (constants)
/// 4. If a block has no defs, it unwraps the block.
/// 5. Check cycles
///
/// when it returns Err(()), the actual errors are in `session`
iter_mut_exprs_in_ast!(clean_up_blocks, ());

impl Expr {
    // the last argument is due to the definition of the macro
    pub fn clean_up_blocks(&mut self, session: &mut LocalParseSession, _: &mut ()) -> Result<(), ASTError> {
        self.kind.clean_up_blocks(session, &mut ())?;

        // in case `clean_up_blocks` removed all the blocks
        if self.is_block_with_0_defs() {
            *self = self.unwrap_block_value();
        }

        Ok(())
    }
}

impl ExprKind {

    // the last argument is due to the definition of the macro
    pub fn clean_up_blocks(&mut self, session: &mut LocalParseSession, ctxt: &mut ()) -> Result<(), ASTError> {
        match self {
            ExprKind::Value(v) => match v {
                ValueKind::Identifier(_, _)
                | ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Char(_)
                | ValueKind::Bytes(_)
                | ValueKind::Object(_) => {},
                ValueKind::List(elements)
                | ValueKind::Tuple(elements)
                | ValueKind::Format(elements)
                | ValueKind::Closure(_, elements) => {
                    for element in elements.iter_mut() {
                        element.clean_up_blocks(session, ctxt)?;
                    }
                },
                ValueKind::Lambda(args, val) => {
                    for ArgDef { ty, .. } in args.iter_mut() {
                        if let Some(ty) = ty {
                            ty.clean_up_blocks(session, ctxt)?;
                        }
                    }

                    val.clean_up_blocks(session, ctxt)?;
                },
                ValueKind::Block { defs, value, id } => {
                    for BlockDef { value, ty, .. } in defs.iter_mut() {
                        value.clean_up_blocks(session, ctxt)?;

                        if let Some(ty) = ty {
                            ty.clean_up_blocks(session, ctxt)?;
                        }
                    }
                    value.clean_up_blocks(session, ctxt)?;

                    // running this pass multiple times can reduce even more blocks!
                    for i in 0..2 {
                        let graph = get_dep_graph(&defs, &value, *id);
                        let mut never_used = vec![];
                        let mut once_used = vec![];

                        if i != 0 {
                            let cycles = find_cycles(&graph);

                            if !cycles.is_empty() {
                                let mut data = cycles.into_iter().map(
                                    |name| (name, get_span_by_name(name, defs))
                                ).collect::<Vec<(InternedString, Span)>>();
                                data.sort_by_key(|(_, span)| span.start);

                                let mut err = ASTError::recursive_def(data.clone());

                                if data.iter().any(|(name, _)| is_closure(name, &defs)) {
                                    err.set_msg(
                                        String::from("Sodigy doesn't implement recursive closures currently.\nIf that's what you want, please try another way.")
                                    );
                                }

                                return Err(err);
                            }
                        }

                        for (def_name, usage) in graph.iter() {

                            if usage.len() == 0 {
                                never_used.push(*def_name);
                            }

                            else if usage.len() == 1 {
                                once_used.push(*def_name);
                            }

                        }

                        // remove `never_used` ones
                        for never_used_name in never_used.iter() {
                            session.add_warning(SodigyWarning::unused_param(*never_used_name, get_span_by_name(*never_used_name, defs), ParamType::BlockDef));

                            defs.swap_remove(
                                defs.iter().position(
                                    |BlockDef { name, .. }| name == never_used_name
                                ).expect("Internal Compiler Error 3535BE1925D")
                            );
                        }

                        for BlockDef { value, name, .. } in defs.iter() {
                            if is_simple_expr(value) {
                                once_used.push(*name);
                            }
                        }

                        // remove duplicates
                        let once_used_or_simple: HashSet<InternedString> = once_used.into_iter().collect();

                        // HashMap<from, to>
                        let mut substitutions: HashMap<(InternedString, NameOrigin), Expr> = HashMap::with_capacity(once_used_or_simple.len());

                        // substitute `once_used` ones and remove their defs
                        // substitute `simple` ones and remove their defs
                        for name_to_subs in once_used_or_simple.iter() {
                            let ind = defs.iter().position(
                                |BlockDef { name, .. }| name == name_to_subs
                            ).expect("Internal Compiler Error E7C812E19B6");
                            let BlockDef { value: value_to_subs, .. } = defs[ind].clone();
                            substitutions.insert((*name_to_subs, NameOrigin::BlockDef(*id)), value_to_subs);

                            defs.swap_remove(ind);
                        }

                        let subst_names = substitutions.keys().map(|ns| ns.clone()).collect::<Vec<_>>();

                        // it has to be done because there may be recursive substitutions
                        // e.g. (a -> 3), (b -> a), (c -> a) we have to make sure that `b` becomes `3`, not `a`.
                        for subst_name in subst_names.iter() {
                            // borrowck...
                            let mut subst_val = substitutions.get_mut(subst_name).unwrap().clone();
                            substitute_local_def(&mut subst_val, &substitutions);
                            *substitutions.get_mut(subst_name).unwrap() = subst_val;
                        }

                        for BlockDef { value, .. } in defs.iter_mut() {
                            substitute_local_def(value, &substitutions);
                        }

                        substitute_local_def(value, &substitutions);
                    }
                },
            },
            ExprKind::Prefix(_, v) => v.clean_up_blocks(session, ctxt)?,
            ExprKind::Postfix(_, v) => v.clean_up_blocks(session, ctxt)?,
            ExprKind::Infix(_, v1, v2) => {
                v1.clean_up_blocks(session, ctxt)?;
                v2.clean_up_blocks(session, ctxt)?;
            },
            ExprKind::Match(value, branches, _) => {
                value.clean_up_blocks(session, ctxt)?;

                for MatchBranch { value, .. } in branches.iter_mut() {
                    value.clean_up_blocks(session, ctxt)?;
                }
            },
            ExprKind::Branch(c, t, f) => {
                c.clean_up_blocks(session, ctxt)?;
                t.clean_up_blocks(session, ctxt)?;
                f.clean_up_blocks(session, ctxt)?;
            },
            ExprKind::Call(f, args) => {
                f.clean_up_blocks(session, ctxt)?;

                for arg in args.iter_mut() {
                    arg.clean_up_blocks(session, ctxt)?;
                }

            }
        }

        Ok(())
    }

}

// HashMap<K, Vec<K>>, where K is a name of a local-def
// Vec<K> stores usage of the key.
// if hash_map[foo] = [bar, bar, InternedString::dummy()], that means `foo` is used in `bar` twice and in the main value once
fn get_dep_graph(defs: &Vec<BlockDef>, value: &Box<Expr>, id: UID) -> HashMap<InternedString, Vec<InternedString>> {
    let mut result = HashMap::with_capacity(defs.len());

    for BlockDef { name: name1, .. } in defs.iter() {
        let mut occurrence = vec![];

        for BlockDef { name: name2, value, ty, .. } in defs.iter() {
            let mut count = 0;
            count_occurrence(value, *name1, id, &mut count);

            if let Some(ty) = ty {
                count_occurrence(ty, *name1, id, &mut count);
            }

            for _ in 0..count {
                occurrence.push(*name2);
            }
        }

        let mut count = 0;
        count_occurrence(value, *name1, id, &mut count);

        for _ in 0..count {
            occurrence.push(InternedString::dummy());
        }

        result.insert(*name1, occurrence);
    }

    result
}

fn count_occurrence(expr: &Expr, name: InternedString, block_id: UID, count: &mut usize) {
    match &expr.kind {
        ExprKind::Value(v) => match v {
            ValueKind::Identifier(name_, NameOrigin::BlockDef(id)) if *name_ == name && *id == block_id => {
                *count += 1;
            },
            ValueKind::Identifier(_, _)
            | ValueKind::Integer(_)
            | ValueKind::Real(_)
            | ValueKind::String(_)
            | ValueKind::Char(_)
            | ValueKind::Bytes(_)
            | ValueKind::Object(_) => {},
            ValueKind::Format(elements)
            | ValueKind::List(elements)
            | ValueKind::Tuple(elements)
            | ValueKind::Closure(_, elements) => {
                for element in elements.iter() {
                    count_occurrence(element, name, block_id, count);
                }
            },
            ValueKind::Lambda(args, value) => {
                count_occurrence(value.as_ref(), name, block_id, count);

                for ArgDef { ty, .. } in args.iter() {
                    if let Some(ty) = ty {
                        count_occurrence(ty, name, block_id, count);
                    }
                }

            },
            ValueKind::Block { defs, value, .. } => {
                count_occurrence(value, name, block_id, count);

                for BlockDef { value, ty, .. } in defs.iter() {
                    count_occurrence(value, name, block_id, count);

                    if let Some(ty) = ty {
                        count_occurrence(ty, name, block_id, count);
                    }
                }
            }
        },
        ExprKind::Prefix(_, op) | ExprKind::Postfix(_, op) => {
            count_occurrence(op, name, block_id, count);
        },
        ExprKind::Infix(_, op1, op2) => {
            count_occurrence(op1, name, block_id, count);
            count_occurrence(op2, name, block_id, count);
        },
        ExprKind::Match(value, branches, _) => {
            count_occurrence(value, name, block_id, count);

            // it only counts BlockDef(id), which doesn't appear inside patterns
            for MatchBranch { value, .. } in branches.iter() {
                count_occurrence(value, name, block_id, count);
            }
        }
        ExprKind::Branch(c, t, f) => {
            count_occurrence(c, name, block_id, count);
            count_occurrence(t, name, block_id, count);
            count_occurrence(f, name, block_id, count);
        },
        ExprKind::Call(f, args) => {
            count_occurrence(f, name, block_id, count);

            for arg in args.iter() {
                count_occurrence(arg, name, block_id, count);
            }
        },
    }
}

fn is_simple_expr(e: &Expr) -> bool {

    match e.kind {
        // TODO: anything else?
        ExprKind::Value(ValueKind::Identifier(_, _)) => true,
        _ => false
    }

}

// It returns all the nodes that are in cycles
fn find_cycles(graph: &HashMap<InternedString, Vec<InternedString>>) -> Vec<InternedString> {
    let mut cycles = vec![];

    for (node, succ) in graph.iter() {
        let mut visited = HashSet::with_capacity(graph.len());
        let mut stack = vec![];

        for succ_node in succ.iter() {
            visited.insert(*succ_node);
            stack.push(*succ_node);
        }

        while let Some(n) = stack.pop() {

            if n.is_dummy() {
                continue;
            }

            for succ in graph.get(&n).expect("Internal Compiler Error 234AA43FC08").iter() {
                if !visited.contains(succ) {
                    visited.insert(*succ);
                    stack.push(*succ);
                }
            }

        }

        if visited.contains(node) {
            cycles.push(*node);
        }

    }

    cycles
}

fn get_span_by_name(name: InternedString, defs: &Vec<BlockDef>) -> Span {
    for BlockDef { name: name_, span, .. } in defs.iter() {
        if *name_ == name {
            return *span;
        }
    }
    panic!("Internal Compiler Error D679C5FEB5E");
}

fn dump_graph(graph: &HashMap<InternedString, Vec<InternedString>>, session: &LocalParseSession) -> String {
    let mut buf = Vec::with_capacity(graph.len());

    for (key, val) in graph.iter() {
        buf.push(format!(
            "{}: [{}]",
            key.to_string(session),
            val.iter().map(|v| v.to_string(session)).collect::<Vec<String>>().join(", "),
        ));
    }

    format!("({})", buf.join(", "))
}

fn dump_substitutions(sbst: &HashMap<(InternedString, NameOrigin), Expr>, session: &LocalParseSession) -> String {
    let mut buf = Vec::with_capacity(sbst.len());

    for ((name, origin), val) in sbst.iter() {
        buf.push(format!(
            "({}, {}): {}",
            name.to_string(session),
            "TODO",
            val.dump(session),
        ));
    }

    format!("({})", buf.join(", "))
}

// it's used to tell programmers that
// Sodigy doesn't implement recursive closures
fn is_closure(name: &InternedString, defs: &Vec<BlockDef>) -> bool {
    for BlockDef { name: name_, value, .. } in defs.iter() {
        if name == name_ {
            return value.is_closure();
        }
    }

    false
}
