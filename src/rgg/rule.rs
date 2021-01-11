use crate::rgg::{Node, Value};
use gamma::graph::{AppendableGraph, DefaultGraph};
use std::collections::HashMap;

pub trait HasId {
    fn get_id(&self) -> i32;
}

pub enum Condition {
    Equals(Value),
    LessThan(Value),
    GreaterThan(Value),
    LessThanOrEquals(Value),
    GreaterThanOrEquals(Value),
    /// Greater than Range.0, less than Range.1. Inclusive lower, exclusive upper.
    Range(Value, Value),
}

///
pub enum PostCondition {
    Exact(Value),
}

/// Identify a node to match against
pub struct FromNode {
    /// Identify the node in the context of a rule
    pub id: i32,
    /// Identify the "name" of the node. Optional.
    pub name: Option<String>,
    /// Specify any potential values the node has.
    pub values: HashMap<String, Condition>,
}

impl FromNode {
    /// Check whether the node can match the provided node.
    pub fn match_node(&self, node: &Node) -> bool {
        // If name is specified, needs to match.
        if let Some(name) = self.name.as_ref() {
            if *name != node.name {
                return false;
            }
        }

        // If any values are specified, need to match conditions.
        // TODO

        true
    }
}

impl HasId for FromNode {
    fn get_id(&self) -> i32 {
        self.id
    }
}

/// Identify the output node
pub struct ToNode {
    /// Identify the node in the context of a rule
    pub id: i32,
    /// Identify the "name" of the node. Optional.
    pub name: Option<String>,
    /// Specify any potential values the node has.
    pub values: HashMap<String, PostCondition>,
}

impl HasId for ToNode {
    fn get_id(&self) -> i32 {
        self.id
    }
}

/// A defined node in a ruleset. Has an optional name, and may have edge connections.
pub struct NodeSet<T: HasId> {
    pub nodes: Vec<T>,
    pub edges: Vec<(i32, i32)>,
}

impl<T: HasId> NodeSet<T> {
    /// Encode the node & edge relations as a graph
    pub fn as_graph(&self) -> DefaultGraph {
        let mut graph = DefaultGraph::new();
        for node in &self.nodes {
            graph
                .add_node_with(node.get_id() as usize)
                .unwrap_or_else(|e| panic!("{:?}", e));
        }

        for edge in &self.edges {
            graph
                .add_edge(edge.0 as usize, edge.1 as usize)
                .unwrap_or_else(|e| panic!("{:?}", e));
        }

        graph
    }
}

/// Describes a replacement rule.
pub struct Rule {
    pub from: NodeSet<FromNode>,
    pub to: NodeSet<ToNode>,
}
