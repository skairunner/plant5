use crate::rgg::procedures::{ApplyResult, Procedure};
use crate::rgg::rgg_graph::RggGraph;
use crate::rgg::{Condition, Node};
use gamma::graph::{AppendableGraph, DefaultGraph};
use serde::Deserialize;
use std::collections::HashMap;

pub trait HasId {
    fn get_id(&self) -> i32;
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

/// A defined node in a ruleset. Has an optional name, and may have edge connections.
#[derive(Deserialize)]
pub struct NodeSet {
    pub nodes: Vec<FromNode>,
    #[serde(default)]
    pub edges: Vec<(i32, i32)>,
}

impl NodeSet {
    /// Encode the node & edge relations as a graph
    pub fn as_graph(&self) -> DefaultGraph {
        let mut graph = DefaultGraph::new();
        for node in &self.nodes {
            graph
                .add_node_with(node.id as usize)
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
#[derive(Deserialize)]
pub struct Rule {
    pub from: NodeSet,
    pub to: Vec<Procedure>,
}

/// Tracks the results of executing the entire rule
pub struct RuleResult {
    pub removed: Vec<usize>,
    pub added: Vec<usize>,
    pub modified: Vec<usize>,
}

impl RuleResult {
    pub fn new() -> Self {
        Self {
            removed: Vec::new(),
            added: Vec::new(),
            modified: Vec::new(),
        }
    }
    pub fn add_apply_result(&mut self, apply: ApplyResult) {
        match apply {
            ApplyResult::Removed(r) => self.removed.extend(r),
            ApplyResult::Added(a) => self.added.push(a),
            ApplyResult::Modified(m) => self.modified.push(m),
            _ => {}
        }
    }

    pub fn add(&mut self, result: Self) {
        self.removed.extend(result.removed);
        self.added.extend(result.added);
        self.modified.extend(result.modified);
    }
}

impl Rule {
    /// Find all match and apply the rule to each match.
    /// If a node or edge disappears during applying a rule, it is skipped.
    pub fn apply(&self, graph: &mut RggGraph) -> RuleResult {
        let matches = self.matches(graph).collect::<Vec<_>>();
        let mut result = RuleResult::new();
        for mut mapping in matches {
            if self.check_procedure_targets_exist(&mapping) {
                for procedure in &self.to {
                    let apply_result = procedure.apply(graph, &mut mapping);
                    result.add_apply_result(apply_result);
                }
            } else {
                log::info!("Some targets for Rule apply did not exist and were skipped.");
            }
        }

        result
    }

    /// Check that all procedure targets exist before attempting to run any procedure.
    fn check_procedure_targets_exist(&self, mapping: &HashMap<i32, usize>) -> bool {
        for proc in &self.to {
            if !proc.targets_exist(mapping) {
                return false;
            }
        }
        true
    }
}
