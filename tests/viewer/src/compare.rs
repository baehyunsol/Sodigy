use crate::{Color, Graphic, Id, Node, Session};
use std::collections::hash_map::{Entry, HashMap};

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
        let leaves_a: Vec<(Vec<String>, Option<Color>, Id)> = self.get_leaves(node_a.id, &node_a.children);
        let leaves_b: Vec<(Vec<String>, Option<Color>, Id)> = self.get_leaves(node_b.id, &node_b.children);
        let leaves_a_map: HashMap<&Vec<String>, (Option<Color>, Id)> = leaves_a.iter().map(
            |(path, flag, id)| (path, (*flag, *id))
        ).collect();
        let leaves_b_map: HashMap<&Vec<String>, (Option<Color>, Id)> = leaves_b.iter().map(
            |(path, flag, id)| (path, (*flag, *id))
        ).collect();

        let mut only_in_a: Vec<(&Vec<String>, Id)> = vec![];
        let mut only_in_b: Vec<(&Vec<String>, Id)> = vec![];
        let mut flag_change: HashMap<(Option<Color>, Option<Color>), Vec<(&Vec<String>, Id, Id)>> = HashMap::new();

        for (path, flag_a, id_a) in leaves_a.iter() {
            match leaves_b_map.get(path) {
                Some((flag_b, id_b)) if flag_a != flag_b => match flag_change.entry((*flag_a, *flag_b)) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push((path, *id_a, *id_b));
                    },
                    Entry::Vacant(e) => {
                        e.insert(vec![(path, *id_a, *id_b)]);
                    },
                },
                None => {
                    only_in_a.push((path, *id_a));
                },
                _ => {},
            }
        }

        for (path, _, id_b) in leaves_b.iter() {
            if !leaves_a_map.contains_key(path) {
                only_in_b.push((path, *id_b));
            }
        }

        if !only_in_a.is_empty() {
            let only_in_a_children: Vec<_> = only_in_a.iter().map(
                |(path, id)| (join_path(path), *id)
            ).collect();

            let child_id = Id::random();
            nodes.push(Node {
                id: child_id,
                title: format!("in {title_a}, but not in {title_b}"),
                flag: None,
                metadata: None,
                children: Some(only_in_a_children),
            });
            children.push((format!("only in {title_a}"), child_id));
        }

        if !only_in_b.is_empty() {
            let only_in_b_children: Vec<_> = only_in_b.iter().map(
                |(path, id)| (join_path(path), *id)
            ).collect();

            let child_id = Id::random();
            nodes.push(Node {
                id: child_id,
                title: format!("in {title_b}, but not in {title_a}"),
                flag: None,
                metadata: None,
                children: Some(only_in_b_children),
            });
            children.push((format!("only in {title_b}"), child_id));
        }

        let mut changed_flags: Vec<_> = flag_change.keys().collect();
        changed_flags.sort();

        for (flag_1, flag_2) in changed_flags.iter() {
            let mut change_children = vec![];

            for (path, id_a, id_b) in flag_change.get(&(*flag_1, *flag_2)).unwrap().iter() {
                let comp_node_id = Id::random();
                nodes.push(Node {
                    id: comp_node_id,
                    title: join_path(path),
                    flag: None,
                    metadata: None,
                    children: Some(vec![
                        (title_a.to_string(), *id_a),
                        (title_b.to_string(), *id_b),
                    ]),
                });
                change_children.push((join_path(path), comp_node_id));
            }

            let child_id = Id::random();
            nodes.push(Node {
                id: child_id,
                title: format!("{title_a} {} -> {title_b} {}", title_markup(*flag_1), title_markup(*flag_2)),
                flag: None,
                metadata: None,
                children: Some(change_children),
            });
            children.push((
                format!("different flag ({} -> {})", title_markup(*flag_1), title_markup(*flag_2)),
                child_id,
            ));
        }

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

    pub fn get_leaves(&mut self, id: Id, children: &Option<Vec<(String, Id)>>) -> Vec<(Vec<String>, Option<Color>, Id)> {
        match self.leaves.get(&id) {
            Some(ls) => ls.to_vec(),
            None => match children {
                Some(children) => {
                    let mut result = vec![];

                    for (title, child) in children.iter() {
                        let (_, child_flag) = self.title.get(child).unwrap();
                        let child_flag = *child_flag;
                        let grandchildren: Option<Vec<_>> = self.children.get(child).unwrap().clone();
                        let leaves = self.get_leaves(*child, &grandchildren);

                        if leaves.is_empty() {
                            result.push((vec![title.to_string()], child_flag, *child));
                        } else {
                            for (mut path, flag, leaf) in leaves.into_iter() {
                                path.insert(0, title.to_string());
                                result.push((path, flag, leaf));
                            }
                        }
                    }

                    self.leaves.insert(id, result.to_vec());
                    result
                },
                None => {
                    self.leaves.insert(id, vec![]);
                    vec![]
                },
            },
        }
    }
}

fn join_path(path: &[String]) -> String {
    path.iter().map(
        |segment| format!("[([[segment]])]{segment}[([[/segment]])]")
    ).collect::<Vec<_>>().join(" / ")
}

fn title_markup(flag: Option<Color>) -> String {
    match flag {
        Some(flag) => format!("[([[{flag:?}-flag]])]").to_ascii_lowercase(),
        None => String::from("no-flag"),
    }
}

