use super::{SimpleType, StateMachine, StateMachineOrLeaves};
use crate::Session;
use sodigy_span::Span;
use std::collections::HashMap;

impl Session {
    pub fn render_state_machine(&self, state_machine: &StateMachine, name_map: &HashMap<Span, String>) -> String {
        self.render_state_machine_inner(state_machine, name_map, 1)
    }
}

pub(crate) trait RenderStateMachine {
    fn span_to_string_impl(&self, span: &Span) -> Option<String>;

    fn render_state_machine_inner(&self, state_machine: &StateMachine, name_map: &HashMap<Span, String>, indent: usize) -> String {
        fn render_leaves(l: &[Span], name_map: &HashMap<Span, String>) -> String {
            format!("[{}]", l.iter().map(|s| name_map.get(s).unwrap().to_string()).collect::<Vec<_>>().join(", "))
        }

        let mut arms = vec![];
        let indent_p = "    ".repeat(indent - 1);
        let indent_s = "    ".repeat(indent);

        for (condition, branch) in state_machine.branches.iter() {
            arms.push(format!(
                "\n{indent_s}{} => {},",
                self.render_simple_type(condition),
                match branch {
                    StateMachineOrLeaves::StateMachine(s) => self.render_state_machine_inner(&s, name_map, indent + 1),
                    StateMachineOrLeaves::Leaves(leaves) => render_leaves(&leaves, name_map),
                },
            ));
        }

        arms.push(format!(
            "\n{indent_s}_ => {},",
            match &*state_machine.default {
                StateMachineOrLeaves::StateMachine(s) => self.render_state_machine_inner(&s, name_map, indent + 1),
                StateMachineOrLeaves::Leaves(leaves) => render_leaves(&leaves, name_map),
            },
        ));
        let arms = arms.concat();
        format!(
            "match {} {{{arms}\n{indent_p}}}",
            self.span_to_string_impl(&state_machine.generic_param).unwrap_or(String::from("????")),
        )
    }

    fn render_simple_type(&self, t: &SimpleType) -> String {
        match t {
            SimpleType::Data { constructor, arity } => format!("Data({}, {arity})", self.span_to_string_impl(&Span::Range(*constructor)).unwrap_or(format!("{constructor:?}"))),
            SimpleType::Func { params } => format!("Func({params})"),
            SimpleType::GenericParam => unreachable!(),
            SimpleType::Var => String::from("Var"),
        }
    }
}

impl RenderStateMachine for Session {
    fn span_to_string_impl(&self, span: &Span) -> Option<String> {
        self.span_to_string(span)
    }
}
