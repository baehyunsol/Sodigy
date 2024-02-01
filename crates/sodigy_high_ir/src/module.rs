use crate::attr::Attribute;
use sodigy_ast::IdentWithSpan;
use sodigy_uid::Uid;

pub struct Module {
    pub(crate) name: IdentWithSpan,
    pub(crate) uid: Uid,
    pub(crate) attributes: Vec<Attribute>,
}
