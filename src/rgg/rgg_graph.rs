use crate::rgg::dirty_graph::DirtyGraph;
use crate::rgg::Node;
use gamma::graph::{AppendableGraph, Graph, RemovableGraph};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct RggGraph {
    pub graph: DirtyGraph,
    pub values: HashMap<usize, super::Node>,
}

impl RggGraph {
    pub fn insert_node(&mut self) -> usize {
        let n = self.graph.add_node().unwrap();
        self.values.insert(n, Node::new(""));
        n
    }

    pub fn insert_node_with(&mut self, node: Node) -> usize {
        let n = self.graph.add_node().unwrap();
        self.values.insert(n, node);
        n
    }

    pub fn remove_node(&mut self, id: usize) {
        self.graph.remove_node(id);
        self.values.remove_entry(&id);
    }

    pub fn order(&self) -> usize {
        self.graph.order()
    }

    pub fn neighbors<'a>(
        &'a self,
        id: usize,
    ) -> Result<Box<dyn Iterator<Item = &'a usize> + 'a>, gamma::graph::Error> {
        self.graph.neighbors(id)
    }

    /// Output a DOT-compatible string
    pub fn as_dot_string(&self) -> String {
        let mut strings = vec!["graph {".to_string()];
        for id in self.graph.nodes() {
            let node = self.values.get(id);
            let name = match node {
                Some(node) => node.name.as_str(),
                None => "",
            };
            strings.push(format!(r#"  {} [label="{}"]"#, *id, name));
        }
        for (from, to) in self.graph.edges() {
            strings.push(format!(r#"  {} -- {}"#, *from, *to));
        }
        strings.push("}".to_string());
        strings.join("\n")
    }
}
