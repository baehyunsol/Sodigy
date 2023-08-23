use sdg_ast::{
    AST, FuncDef,
    InternedString,
    LocalParseSession,
    FuncKind,
};
use sdg_uid::UID;
use std::collections::HashMap;

pub enum ModuleOrDef {
    Def(UID),

    /// names of child modules or defs
    Module(HashMap<InternedString, ModuleOrDef>),

    /// name of variants -> UID of variant defs\
    /// the first uid -> uid of the enum
    Enum(UID, HashMap<InternedString, UID>),
}

impl ModuleOrDef {
    pub fn dump(&self, session: &LocalParseSession) -> String {
        match self {
            ModuleOrDef::Def(uid) => format!("Def({})", uid.to_string()),
            ModuleOrDef::Module(names) => {
                format!("Module({})", dump_module(names, session))
            },
            ModuleOrDef::Enum(uid, uids) => {
                let hashmap = uids.iter().map(
                    |(name, uid)| format!(
                        "{}: {}",
                        name.to_string(session),
                        uid.to_string(),
                    )
                ).collect::<Vec<String>>().join(", ");
                format!("Enum({}, {{{hashmap}}})", uid.to_string())
            },
        }
    }
}

pub fn dump_module(module: &HashMap<InternedString, ModuleOrDef>, session: &LocalParseSession) -> String {
    let elements = module.iter().map(
        |(name, content)| format!(
            "{}: {}",
            name.to_string(session),
            content.dump(session),
        )
    ).collect::<Vec<String>>().join(", ");

    format!("{{{elements}}}")
}

/// it has information of all the `FuncDef`s in the source.
/// the `FuncDef`s are indexed by `UID`s.
pub struct InterModuleContext {
    pub namespace: HashMap<InternedString, ModuleOrDef>,
    func_defs: HashMap<UID, FuncDef>,
}

impl InterModuleContext {
    pub fn new() -> Self {
        InterModuleContext {
            namespace: HashMap::new(),
            func_defs: HashMap::new(),
        }
    }

    pub fn collect_ast(&mut self, ast: &AST) {
        for (name, def) in ast.defs.iter() {
            // TODO: cloning would be too expensive!
            // TODO: check collision
            self.func_defs.insert(def.id, def.clone());

            let path = def.get_full_path().to_names();
            insert_path(&mut self.namespace, &path, def.id, &def.kind);
        }
    }

    pub fn dump(&self, session: &mut LocalParseSession) -> String {
        // see the comment in `ast::dump` to see what
        // `uid_to_name_table` does
        let mut uid_to_name_table = HashMap::new();

        for (id, def) in self.func_defs.iter() {
            uid_to_name_table.insert(
                *id,
                def.pretty_name(session),
            );
        }

        session.update_uid_to_name_table(uid_to_name_table);
        session.update_prelude_uid_table();

        format!(
            "namespace: {}, func_defs: {{{}}}",
            dump_module(&self.namespace, session),
            self.func_defs.iter().map(
                |(id, def)| format!(
                    "{}: {}",
                    id.to_u128(),
                    def.dump(session),
                )
            ).collect::<Vec<String>>().join(", "),
        )
    }
}

fn insert_path(curr: &mut HashMap<InternedString, ModuleOrDef>, path: &[InternedString], uid: UID, func_kind: &FuncKind) {
    match curr.get_mut(&path[0]) {
        Some(ModuleOrDef::Module(recur)) => {
            insert_path(recur, &path[1..], uid, func_kind);
        },
        Some(ModuleOrDef::Enum(curr_uid, names)) => {
            if path.len() == 1 {  // enum
                assert_eq!(*curr_uid, UID::dummy(), "Internal Compiler Error 4E396E1AC21");
                *curr_uid = uid;
            } else {  // enum variant
                assert_eq!(path.len(), 2, "Internal Compiler Error 0ABA4C29A87");

                names.insert(path[1], uid);
            }
        },
        Some(ModuleOrDef::Def(uid)) => {
            // full path must be unique
            unreachable!("Internal Compiler Error 854EC3E3655");
        },
        None => {
            if func_kind.is_enum_var() && path.len() == 2 {
                let mut names = HashMap::new();
                names.insert(path[1], uid);
                curr.insert(path[0], ModuleOrDef::Enum(UID::dummy(), names));
            } else if func_kind.is_enum_def() && path.len() == 1 {
                curr.insert(path[0], ModuleOrDef::Enum(uid, HashMap::new()));
            } else if path.len() == 1 {
                curr.insert(path[0], ModuleOrDef::Def(uid));
            } else {
                let mut recur = HashMap::new();
                insert_path(&mut recur, &path[1..], uid, func_kind);
                curr.insert(path[0], ModuleOrDef::Module(recur));
            }
        },
    }
}
