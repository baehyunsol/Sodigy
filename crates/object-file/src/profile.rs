use crate::{Assert, AssertionFilter};

#[derive(Clone, Debug)]
pub enum Profile {
    Run,
    Test {
        std_assertions: bool,
        filters: Option<Vec<AssertionFilter>>,
    },
}

impl Profile {
    pub fn has_to_check_this_assertion(&self, assertion: &Assert) -> bool {
        match (self, assertion.is_std, &assertion.name) {
            (Profile::Run, _, _) => false,
            (Profile::Test { std_assertions: false, .. }, true, _) => false,
            (Profile::Test { filters, .. }, _, name) => match (filters, name) {
                (None, _) => true,
                (Some(filters), name) => todo!(),
            },
        }
    }
}

