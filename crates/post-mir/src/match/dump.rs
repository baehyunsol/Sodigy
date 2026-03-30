use super::{
    DecisionTree,
    DecisionTreeNode,
    ExprConstructor,
    NameBindingOffset,
    PatternField,
};
use crate::Session;
use sodigy_endec::IndentedLines;
use sodigy_mir::{MatchArm, Session as MirSession, dump_expr};
use sodigy_span::Span;
use std::collections::hash_map::{Entry, HashMap};

#[derive(Clone, Debug)]
pub struct MatchDump {
    pub keyword_span: Span,
    pub span_helpers: Vec<(Span, String)>,
    pub decision_tree: String,
    pub expr: String,
}

impl Session<'_, '_> {
    pub fn dump_decision_tree(
        &self,
        tree: &DecisionTree,
        arms: &[(usize, &MatchArm)],
    ) -> (Vec<(Span, String)>, String) {
        let mut buffer = vec![];
        let mut span_helpers = HashMap::new();
        self.dump_decision_tree_inner(tree, &mut buffer, &mut span_helpers, 0);
        let mut span_helpers = span_helpers.into_iter().collect::<Vec<_>>();
        span_helpers.extend(arms.iter().map(
            |(id, arm)| (
                arm.value.error_span_wide(),
                format!("arm_{id}"),
            )
        ));
        (span_helpers, buffer.concat())
    }

    fn dump_decision_tree_inner(
        &self,
        tree: &DecisionTree,
        buffer: &mut Vec<String>,
        span_helpers: &mut HashMap<Span, String>,
        indent: usize,
    ) {
        let scrutinee = match &tree.field {
            Some(field) => format!(
                "v{}",
                field.iter().map(|field| self.render_pattern_field(field)).collect::<Vec<_>>().concat(),
            ),
            None => String::from("_"),
        };
        buffer.push(format!("match {scrutinee} {{\n"));

        for branch in tree.branches.iter() {
            for name_binding in branch.name_bindings.iter() {
                let span_helpers_len = span_helpers.len();
                let nb_index = match span_helpers.entry(name_binding.name_span.clone()) {
                    Entry::Occupied(e) => e.get().to_string(),
                    Entry::Vacant(e) => {
                        let nb_index = format!("nb_{span_helpers_len}");
                        e.insert(nb_index.to_string());
                        nb_index
                    },
                };

                let offset = match &name_binding.offset {
                    NameBindingOffset::None => None,
                    NameBindingOffset::Number(n) => Some(todo!()),
                    NameBindingOffset::Slice(a, b) => Some(format!("slice({a}, {b})")),
                };

                buffer.push("    ".repeat(indent + 1));
                buffer.push(format!(
                    "#[name_binding({}, {nb_index}{})]\n",
                    name_binding.name.unintern_or_default(&self.intermediate_dir),
                    if let Some(offset) = offset { format!(", offset={offset}") } else { String::new() },
                ));
            }

            buffer.push("    ".repeat(indent + 1));
            buffer.push(self.render_expr_constructor(&branch.condition));

            if let Some(guard) = &branch.guard {
                let mut lines = IndentedLines::new();
                let types = self.global_context.types.as_ref().unwrap().as_ref().read().unwrap();
                dump_expr(guard, &mut lines, &types, self, 0, true);
                buffer.push(format!(" if {}", lines.dump()));
            }

            buffer.push(String::from(" => "));

            match &branch.node {
                DecisionTreeNode::Tree(tree) => {
                    self.dump_decision_tree_inner(tree, buffer, span_helpers, indent + 1);
                },
                DecisionTreeNode::Leaf { matched, .. } => {
                    buffer.push(format!("arm_{matched}"));
                },
            }

            buffer.push(String::from(",\n"));
        }

        buffer.push("    ".repeat(indent));
        buffer.push(String::from("}"));
    }

    fn render_pattern_field(&self, field: &PatternField) -> String {
        match field {
            PatternField::Constructor => String::from(".constructor()"),
            PatternField::Name { name, .. } => format!(".{}", name.unintern_or_default(&self.intermediate_dir)),
            PatternField::Index(n) => format!("._{n}"),
            PatternField::ListIndex(n) => format!("[{n}]"),
            PatternField::ListLength => String::from(".len()"),
            PatternField::ListElements => String::from(".elements()"),
            _ => panic!("TODO: {field:?}"),
        }
    }

    fn render_expr_constructor(&self, expr: &ExprConstructor) -> String {
        match expr {
            ExprConstructor::Range(r) => r.to_string(),
            ExprConstructor::Or(es) => es.iter().map(
                |e| self.render_expr_constructor(e)
            ).collect::<Vec<_>>().join(" | "),
            ExprConstructor::Wildcard => String::from("_"),
        }
    }
}
