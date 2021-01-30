use std::collections::HashMap;

use super::Value;
use crate::rgg::Condition;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// Represents the values stored in a node in an RGG.
pub struct Node {
    pub name: String,
    #[serde(default)]
    pub values: HashMap<String, Value>,
}

impl Node {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: Default::default(),
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new("")
    }
}

/// Identify a node to match against
#[derive(Deserialize)]
pub struct FromNode {
    /// Identify the node in the context of a rule
    pub id: i32,
    /// Identify the "name" of the node. Optional.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
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

/// Define a replacement node.
/// For replace, can use operations relative to the previous node's values.
/// For all nodes, can use some operations for values, such as rand
#[derive(Deserialize)]
pub struct ToNode {
    pub name: String,
    pub values: HashMap<String, String>,
}

impl ToNode {
    /// Evaluate the values of the tonode to create a normal node
    pub fn eval(&self, base_node: &Node) -> Node {
        Node::new("")
    }
}
