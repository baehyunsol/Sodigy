use crate::{Color, Graphic, Id, Node, Session};
use std::collections::hash_map::Entry;

impl Session {
    pub fn compare_nodes(
        &mut self,
        title_a: &str,
        title_b: &str,
        node_a: &Node,
        node_b: &Node,
        result_id: Id,
        swap_id: Id,
        nodes: &mut Vec<Node>,
    ) {
        let mut children = vec![];
        let leaves_a = self.get_leaves(node_a.id, &node_a.children);
        let leaves_b = self.get_leaves(node_b.id, &node_b.children);

        nodes.push(Node {
            id: result_id,
            title: format!("{title_a} vs {title_b}"),
            flag: None,
            metadata: Some(vec![Graphic::List(vec![
                Graphic::Link { text: format!("{title_b} vs {title_a}"), id: swap_id },
                Graphic::Link { text: title_a.to_string(), id: node_a.id },
                Graphic::Link { text: title_b.to_string(), id: node_b.id },
            ])]),
            children: Some(children),
        });
    }

    pub fn get_leaves(&mut self, id: Id, children: &Option<Vec<(String, Id)>>) -> Vec<(Vec<String>, Option<Color>)> {
        match self.leaves.entry(id) {
            Entry::Occupied(e) => e.get().to_vec(),
            Entry::Vacant(e) => {
                let mut result = vec![];

                todo!();

                e.insert(result.to_vec());
                result
            },
        }
    }
}

