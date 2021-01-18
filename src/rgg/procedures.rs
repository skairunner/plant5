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
    #[serde(rename = "replace")]
    pub replacement: Node,
}

#[derive(Deserialize, Debug)]
pub struct AddProcedure {
    /// All the nodes that this new node should have an edge to
    pub neighbors: Vec<i32>,
    #[serde(rename = "node")]
    pub new_node: Node,
}

#[derive(Deserialize, Debug)]
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
    /// Return Some(node id) if the node is dirty, otherwise return None.
    fn get_if_not_dirty(
        target: i32,
        graph: &RggGraph,
        mapping: &HashMap<i32, usize>,
    ) -> CheckDirty {
        match mapping.get(&target) {
            None => CheckDirty::DoesNotExist,
            Some(target) => match graph.graph.node_is_dirty(*target) {
                true => CheckDirty::Dirty,
                false => CheckDirty::Clean(*target),
            },
        }
    }

    /// Apply the contents of the Procedure to a mapped graph.
    pub fn apply(&self, graph: &mut RggGraph, mapping: &mut HashMap<i32, usize>) {
        match self {
            Procedure::Delete(proc) => match Self::get_if_not_dirty(proc.target, graph, mapping) {
                CheckDirty::Dirty => {}
                CheckDirty::DoesNotExist => {
                    log::warn!("Tried to delete a node that was already deleted")
                }
                CheckDirty::Clean(target) => {
                    graph.remove_node(target);
                    mapping.remove(&proc.target);
                }
            },
            Procedure::Replace(proc) => match Self::get_if_not_dirty(proc.target, graph, mapping) {
                CheckDirty::Dirty => {}
                CheckDirty::DoesNotExist => {
                    log::warn!("Tried to replace a node that does not exist")
                }
                CheckDirty::Clean(target) => {
                    graph.values.insert(target, proc.replacement.clone());
                }
            },
            Procedure::Add(proc) => {
                let node_id = graph.insert_node_with(proc.new_node.clone());
                for neighbor in &proc.neighbors {
                    match mapping.get(&neighbor) {
                        Some(neighbor) => {
                            graph.graph.add_edge(node_id, *neighbor).unwrap();
                        }
                        None => log::warn!(
                            "Could not find specified neighbor {} in mapping {:?}",
                            neighbor,
                            mapping
                        ),
                    }
                }
            }
            Procedure::Merge(proc) => {
                // Make a list of all edges that connect to all neighbors
                let mut neighbors: HashSet<usize> = HashSet::new();
                // Ensure that all nodes to be merged are not dirty.
                for rule_id in &proc.targets {
                    match Self::get_if_not_dirty(*rule_id, graph, mapping) {
                        CheckDirty::Dirty => return,
                        CheckDirty::DoesNotExist => {
                            log::warn!("Tried to merge a node that does not exist");
                            return;
                        }
                        CheckDirty::Clean(id) => {
                            graph
                                .graph
                                .neighbors(id)
                                .expect("Could not unwrap neighbors()")
                                .for_each(|n| {
                                    neighbors.insert(*n);
                                });
                        }
                    }
                }
                // Remove all affected nodes, then re-add all the required edges
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
        };
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
}
