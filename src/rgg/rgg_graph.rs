use crate::rgg::dirty_graph::DirtyGraph;
use crate::rgg::Node;
use gamma::graph::{AppendableGraph, RemovableGraph};
use std::collections::HashMap;

pub struct RggGraph {
    pub graph: DirtyGraph,
    pub values: HashMap<usize, super::Node>,
}

impl RggGraph {
    pub fn new() -> Self {
        Self {
            graph: DirtyGraph::new(),
            values: HashMap::new(),
        }
    }

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
}
