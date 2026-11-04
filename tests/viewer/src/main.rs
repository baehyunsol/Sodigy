use sodigy_fs_api::{basename, is_dir, read_dir, read_string};
use viewer::{Color, Graphic, Id, Node, render_all};

fn main() {
    sample();
}

// renders a page with files
fn sample() {
    let mut nodes = vec![];
    let mut id = 0;
    let index_id = collect(".", &mut id, &mut nodes);
    render_all(&nodes, Id::from_u64(index_id), "sample/").unwrap();
}

fn collect(path: &str, id: &mut u64, nodes: &mut Vec<Node>) -> u64 {
    let children = if is_dir(path) {
        let mut children = vec![];

        for entry in read_dir(path, true).unwrap() {
            children.push((basename(&entry).unwrap(), Id::from_u64(collect(&entry, id, nodes))));
        }

        Some(children)
    } else {
        None
    };

    *id += 1;

    let node = if is_dir(path) {
        Node {
            id: Id::from_u64(*id),
            title: path.to_string(),
            flag: Some(Color::Green),
            metadata: None,
            children,
        }
    } else {
        let metadata = match read_string(path) {
            Ok(s) if s.len() < 4096 => Some(vec![Graphic::TextBox(vec![(None, s)])]),
            _ => None,
        };

        Node {
            id: Id::from_u64(*id),
            title: path.to_string(),
            flag: Some(Color::Red),
            metadata: metadata,
            children,
        }
    };

    nodes.push(node);
    *id
}

