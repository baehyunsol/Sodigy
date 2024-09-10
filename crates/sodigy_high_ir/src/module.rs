use crate::attr::Attribute;
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;

mod endec;

pub struct Module {
    pub(crate) name: IdentWithSpan,
    pub(crate) uid: Uid,
    pub(crate) attributes: Vec<Attribute>,
}
