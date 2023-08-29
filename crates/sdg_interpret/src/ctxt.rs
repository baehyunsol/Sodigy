use crate::builtins::BuiltIns;
use crate::typeck::type_check;
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
}

impl TypeCkCtxt {
    pub fn get_type_of_identifier(&self, name: InternedString, origin: NameOrigin) -> Expr {
        todo!()
    }

    pub fn register_block_defs(
        &mut self,
        block_def: &BlockDef,
        block_id: UID,
        session: &mut LocalParseSession,
        funcs: &InterModuleContext,
    ) -> Result<(), ()> {
        let block_def_type = type_check(&block_def.value, session, funcs, self)?;

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
