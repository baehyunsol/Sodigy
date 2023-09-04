use crate::builtins::BuiltIns;
use crate::typeck::type_check_expr;
use sdg_ast::{BlockDef, Expr, InternedString, LocalParseSession, NameOrigin, SodigyError};
use sdg_inter_mod::InterModuleContext;
use sdg_uid::UID;
use std::collections::HashMap;
use std::rc::Rc;

pub struct EvalCtxt {
    args: Vec<Rc<Expr>>,
    session: LocalParseSession,
}

impl EvalCtxt {
    pub fn set_args(&mut self, args: Vec<Rc<Expr>>) {
        self.args = args;
    }

    pub fn add_error<E: SodigyError + 'static>(&mut self, error: E) {
        self.session.add_error(error);
    }

    pub fn evaluate_identifier(&self, name: InternedString, origin: NameOrigin) -> Result<Expr, ()> {
        todo!()
    }
}

pub struct TypeCkCtxt {
    block_defs: HashMap<(UID, InternedString), Expr>,
    curr_func_args: HashMap<InternedString, Expr>,
}

impl TypeCkCtxt {
    pub fn new() -> Self {
        TypeCkCtxt {
            block_defs: HashMap::new(),
            curr_func_args: HashMap::new(),
        }
    }

    pub fn register_func_arg(&mut self, name: InternedString, ty: Expr) {
        self.curr_func_args.insert(name, ty);
    }

    pub fn remove_func_args(&mut self) {
        self.curr_func_args = HashMap::new();
    }

    pub fn get_type_of_identifier(&self, name: InternedString, origin: NameOrigin) -> &Expr {
        // all the name errors must be caught very long ago
        match origin {
            NameOrigin::BlockDef(id) => self.block_defs.get(&(id, name)).expect(
                // it must be pushed when the type checker sees a block-expression
                "Internal Compiler Error 4A4FD5F963A"
            ),
            NameOrigin::FuncArg(_) => self.curr_func_args.get(&name).expect(
                // it must be pushed when the type checker iterates AST
                "Internal Compiler Error 29157754095"
            ),
            _ => todo!(),
        }
    }

    pub fn register_block_defs(
        &mut self,
        block_def: &BlockDef,
        block_id: UID,
        session: &mut LocalParseSession,
        funcs: &InterModuleContext,
    ) -> Result<(), ()> {
        let block_def_type = type_check_expr(&block_def.value, session, funcs, self)?;

        if let Some(ty) = &block_def.ty {
            if !block_def_type.is_subtype_of(ty) {
                // TODO: Err
            }
        }

        self.block_defs.insert((block_id, block_def.name), block_def_type);

        Ok(())
    }

    pub fn drop_block_defs(&mut self, id: UID) {
        todo!()
    }
}
