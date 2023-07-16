use super::super::AST;
use crate::err::ParseError;
use crate::expr::ExprKind;
use crate::stmt::ArgDef;
use crate::value::ValueKind;

impl AST {

    // 1. If a definition is used only once, the value goes directly to the used place.
    // 2. If a definition is used 0 times, it's removed.
    // 3. If a value of a definition is simple, all the referents are replaced with the value.
    //   - simple value: single identifier (or a path), small number (how small?), static values (contants)
    // 4. If a block has no defs, it unwraps the block.
    // 5. Check cycles?
    pub fn clean_up_blocks(&mut self) -> Result<(), ParseError> {

        for func in self.defs.values_mut() {
            func.ret_val.kind.clean_up_blocks()?;

            if let Some(ty) = &mut func.ret_type {
                ty.kind.clean_up_blocks()?;
            }

            for ArgDef { ty, .. } in func.args.iter_mut() {
                if let Some(ty) = ty {
                    ty.kind.clean_up_blocks()?;
                }
            }

        }

        Ok(())
    }
}

impl ExprKind {

    pub fn clean_up_blocks(&mut self) -> Result<(), ParseError> {
        match self {
            ExprKind::Value(v) => match v {
                ValueKind::Identifier(_, _)
                | ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Bytes(_) => {},
                ValueKind::List(elements)
                | ValueKind::Tuple(elements)
                | ValueKind::Format(elements) => {
                    for element in elements.iter_mut() {
                        element.kind.clean_up_blocks()?;
                    }
                },
                ValueKind::Lambda(args, val) => {
                    for ArgDef { ty, .. } in args.iter_mut() {
                        if let Some(ty) = ty {
                            ty.kind.clean_up_blocks()?;
                        }
                    }

                    val.kind.clean_up_blocks()?;
                },
                ValueKind::Block { defs, value, id } => {
                    let graph = get_dep_graph(&defs, &value, *id);
                    let never_used = vec![];
                    let used_once = vec![];

                    if let Some(name) = find_cycle(&graph) {
                        return Err(ParseError::recursive_def(name));
                    }

                    for (def_name, usage) in graph.iter() {

                        if usage.len() == 0 {
                            never_used.push(*def_name);
                        }

                        else if usage.len() == 1 {
                            used_once.push(*def_name);
                        }

                    }

                    // TODO: remove never_used ones
                    // TODO: substitute used_once ones
                    // TODO: substitute simple ones
                    // TODO: if all the defs are removed, unwrap the block
                },
            },
            ExprKind::Prefix(_, v) => v.kind.clean_up_blocks()?,
            ExprKind::Postfix(_, v) => v.kind.clean_up_blocks()?,
            ExprKind::Infix(_, v1, v2) => {
                v1.kind.clean_up_blocks()?;
                v2.kind.clean_up_blocks()?;
            },
            ExprKind::Branch(c, t, f) => {
                c.kind.clean_up_blocks()?;
                t.kind.clean_up_blocks()?;
                f.kind.clean_up_blocks()?;
            },
            ExprKind::Call(f, args) => {
                f.kind.clean_up_blocks()?;

                for arg in args.iter_mut() {
                    arg.kind.clean_up_blocks()?;
                }

            }
        }

        Ok(())
    }

}