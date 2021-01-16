use crate::rgg::rgg_graph::RggGraph;
use crate::rgg::Node;
use gamma::graph::{AppendableGraph, Graph, RemovableGraph};
use std::collections::{HashMap, HashSet};

/// Rules to follow to go from LHS to RHS
/// CBA to figure out double pushout so i will instead "cheat" by having a procedure to follow
pub enum Procedure {
    Delete(DeleteProcedure),
    Replace(ReplaceProcedure),
    Add(AddProcedure),
    Merge(MergeProcedure),
}

pub struct DeleteProcedure {
    pub target: i32,
}

pub struct ReplaceProcedure {
    pub target: i32,
    pub replacement: Node,
}

pub struct AddProcedure {
    /// All the nodes that this new node should have an edge to
    pub neighbors: Vec<i32>,
    pub new_node: Node,
}

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
                    graph.graph.remove_node(target);
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
                            graph.graph.add_edge(node_id, *neighbor);
                        }
                        None => panic!(
                            "Could not find specified neighbor {} in mapping {:?}",
                            neighbor, mapping
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
                    graph.graph.add_edge(node_id, neighbor);
                }
            }
        };
    }
}
