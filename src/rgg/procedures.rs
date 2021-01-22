use crate::rgg::rgg_graph::RggGraph;
use crate::rgg::Node;

use std::collections::{HashMap, HashSet};

use gamma::graph::{AppendableGraph, Graph, RemovableGraph};
use serde::Deserialize;

/// Rules to follow to go from LHS to RHS
/// CBA to figure out double pushout so i will instead "cheat" by having a procedure to follow
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Procedure {
    Delete(DeleteProcedure),
    Replace(ReplaceProcedure),
    Add(AddProcedure),
    Merge(MergeProcedure),
}

#[derive(Debug)]
pub struct DeleteProcedure {
    pub target: i32,
}

#[derive(Deserialize, Debug)]
pub struct ReplaceProcedure {
    pub target: i32,
    #[serde(rename = "with")]
    pub replacement: Node,
}

#[derive(Deserialize, Debug)]
pub struct AddProcedure {
    /// All the nodes that this new node should have an edge to
    pub neighbors: Vec<i32>,
    #[serde(rename = "node")]
    pub new_node: Node,
}

#[derive(Debug)]
pub struct MergeProcedure {
    /// All the nodes to merge
    pub targets: Vec<i32>,
    /// Which node should remain
    pub final_node: i32,
}

enum CheckDirty {
    Clean(usize),
    DoesNotExist,
    Dirty,
}

impl Procedure {
    /// Check whether all targets specified exist in the mapping.
    pub fn targets_exist(&self, mapping: &HashMap<i32, usize>) -> bool {
        match self {
            Procedure::Delete(proc) => mapping.contains_key(&proc.target),
            Procedure::Replace(proc) => mapping.contains_key(&proc.target),
            Procedure::Add(proc) => {
                for neighbor in &proc.neighbors {
                    if !mapping.contains_key(neighbor) {
                        return false;
                    }
                }
                true
            }
            Procedure::Merge(proc) => {
                for target in &proc.targets {
                    if !mapping.contains_key(target) {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// Apply the contents of the Procedure to a mapped graph.
    /// Returns false on failure to execute.
    pub fn apply(&self, graph: &mut RggGraph, mapping: &mut HashMap<i32, usize>) -> bool {
        match self {
            Procedure::Delete(proc) => match mapping.get(&proc.target) {
                Some(target) => {
                    graph.remove_node(*target);
                    mapping.remove(&proc.target);
                }
                None => {
                    log::error!("Could not delete node {}", proc.target);
                    return false;
                }
            },
            Procedure::Replace(proc) => match mapping.get(&proc.target) {
                Some(target) => {
                    graph.values.insert(*target, proc.replacement.clone());
                }
                None => {
                    log::error!("Could not replace node {}", proc.target);
                    return false;
                }
            },
            Procedure::Add(proc) => {
                let node_id = graph.insert_node_with(proc.new_node.clone());
                for neighbor in &proc.neighbors {
                    match mapping.get(&neighbor) {
                        Some(neighbor) => {
                            graph.graph.add_edge(node_id, *neighbor).unwrap();
                        }
                        None => {
                            log::warn!(
                                "Could not find specified neighbor {} in mapping {:?}",
                                neighbor,
                                mapping
                            );
                            return false;
                        }
                    }
                }
            }
            Procedure::Merge(proc) => {
                // Make a list of all edges that connect to all neighbors
                let mut neighbors: HashSet<usize> = HashSet::new();
                // Ensure that all nodes to be merged exist
                for rule_id in &proc.targets {
                    match mapping.get(rule_id) {
                        Some(id) => {
                            graph
                                .graph
                                .neighbors(*id)
                                .expect("Could not unwrap neighbors()")
                                .for_each(|n| {
                                    neighbors.insert(*n);
                                });
                        }
                        None => {
                            log::error!(
                                "Could not merge because missing mapping for node {}",
                                rule_id
                            );
                            return false;
                        }
                    }
                }
                // Remove all affected nodes, then re-add all the required edges.
                for rule_id in &proc.targets {
                    if *rule_id != proc.final_node {
                        let node_id = mapping[&rule_id];
                        graph.remove_node(node_id);
                    }
                }
                let node_id = mapping[&proc.final_node];
                for neighbor in neighbors {
                    graph.graph.add_edge(node_id, neighbor).unwrap();
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::rgg::rgg_graph::RggGraph;
    use crate::rgg::Node;
    use gamma::graph::{AppendableGraph, Graph};
    use std::collections::HashMap;

    /// Gets a triangle graph with all nodes connected, plus its associated mapping
    fn get_simple_graph() -> (RggGraph, HashMap<i32, usize>) {
        let mut graph = RggGraph::new();
        graph.insert_node();
        graph.insert_node();
        graph.insert_node();
        graph.graph.add_edge(0, 1).unwrap();
        graph.graph.add_edge(1, 2).unwrap();
        graph.graph.add_edge(2, 0).unwrap();
        graph.graph.advance_generation();

        let map = maplit::hashmap! {
            2 => 0,
            1 => 1,
            0 => 2,
        };

        (graph, map)
    }

    #[test]
    fn test_simple_add() {
        let proc = Procedure::Add(AddProcedure {
            neighbors: vec![0, 1],
            new_node: Node::new("newnode"),
        });
        let (mut graph, mut mapping) = get_simple_graph();
        proc.apply(&mut graph, &mut mapping);
        assert_eq!(graph.values[&3].name, "newnode");
        let mut neighbors = graph
            .graph
            .neighbors(3)
            .unwrap()
            .map(|id| *id)
            .collect::<Vec<_>>();
        neighbors.sort();
        assert_eq!(neighbors, vec![1, 2]);
    }

    #[test]
    fn test_simple_delete() {
        let proc = Procedure::Delete(DeleteProcedure { target: 2 });
        let (mut graph, mut mapping) = get_simple_graph();
        proc.apply(&mut graph, &mut mapping);
        assert_eq!(graph.graph.order(), 2, "Contents {:?}", graph.graph);
        assert_eq!(graph.values.len(), 2, "Contents {:?}", graph.values);
    }

    #[test]
    fn test_dont_touch_dirty() {
        let proc = Procedure::Delete(DeleteProcedure { target: 0 });
        let mut graph = RggGraph::new();
        graph.insert_node();
        let mut mapping = maplit::hashmap! { 0 => 0 };
        proc.apply(&mut graph, &mut mapping);
        assert_eq!(graph.graph.order(), 1, "Contents {:?}", graph.graph);
    }
}
