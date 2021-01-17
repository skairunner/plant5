use gamma::graph::{AppendableGraph, Error, Graph, RemovableGraph};
use std::collections::{BTreeMap, BTreeSet};

fn new_edge(node1: usize, node2: usize) -> (usize, usize) {
    if node1 <= node2 {
        (node1, node2)
    } else {
        (node2, node1)
    }
}

#[derive(Debug)]
/// An implementation of the gamma::Graph API that supports associating an integer to nodes and edges.
/// Essentially, this allows keeping track of whether a node/edge was already touched this iteration.
pub struct DirtyGraph {
    /// Store the nodes that exist.
    nodes: BTreeSet<usize>,
    /// Store the edges that exist. Since DirtyGraph is a non-directed graph, we should always
    /// attempt to store edges sorted so that "from" is always less than or equal to "to".
    edges: BTreeSet<(usize, usize)>,
    /// Store adjancency details: the value is all adjacent elements to the nodeid that is the key.
    /// Each edge is effectively stored as two adjacencies: one from A->B and one from B->A.
    adjacency: BTreeMap<usize, Vec<usize>>,
    /// Stores the dirty integer assoc with nodes.
    node_generation: BTreeMap<usize, u8>,
    /// Stores the dirty edge assoc with nodes.
    edge_generation: BTreeMap<(usize, usize), u8>,
    /// The node ID for the next node to be generated.
    next_node: usize,
    /// The generation id for the next series of matchings.
    /// If generation id == 255, we need to reset all generations to 0 once we are done with this generation.
    next_generation: u8,
}

impl DirtyGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
            adjacency: Default::default(),
            node_generation: Default::default(),
            edge_generation: Default::default(),
            next_node: 0,
            next_generation: 1,
        }
    }

    fn add_to_adjacency(&mut self, lhs: usize, rhs: usize) {
        (*self.adjacency.entry(lhs).or_insert_with(|| vec![])).push(rhs)
    }

    /// Infallible internal method used to implement has_edge()
    fn contains_edge(&self, sid: usize, tid: usize) -> bool {
        let edge = new_edge(sid, tid);
        self.edges.contains(&edge)
    }

    /// Increment the generation, resetting if required.
    pub fn advance_generation(&mut self) {
        self.next_generation += 1;
        if self.next_generation == 255 {
            self.node_generation.values_mut().for_each(|mut v| *v = 0);
            self.edge_generation.values_mut().for_each(|mut v| *v = 0);
            self.next_generation = 1;
        }
    }

    /// Set the given node as dirty. Returns false if the node didn't exist.
    pub fn set_node_dirty(&mut self, node: usize) -> bool {
        match self.node_generation.get_mut(&node) {
            Some(gen) => {
                *gen = self.next_generation;
                true
            }
            None => false,
        }
    }

    /// Returns true if the node is dirty, aka the node's generation is the same as
    /// self.next_generation
    pub fn node_is_dirty(&self, node: usize) -> bool {
        if !self.nodes.contains(&node) {
            return false;
        }
        let gen = match self.node_generation.get(&node) {
            Some(gen) => *gen,
            None => return false,
        };

        gen >= self.next_generation
    }
}

impl Graph for DirtyGraph {
    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn order(&self) -> usize {
        self.nodes.len()
    }

    fn size(&self) -> usize {
        self.edges.len()
    }

    fn nodes<'a>(&'a self) -> Box<dyn Iterator<Item = &'a usize> + 'a> {
        Box::new(self.nodes.iter())
    }

    fn neighbors<'a>(
        &'a self,
        id: usize,
    ) -> Result<Box<dyn Iterator<Item = &'a usize> + 'a>, Error> {
        let neighbors = self
            .adjacency
            .get(&id)
            .ok_or(gamma::graph::Error::MissingNode(id))?;
        Ok(Box::new(neighbors.iter()))
    }

    fn has_node(&self, id: usize) -> bool {
        self.nodes.contains(&id)
    }

    fn degree(&self, id: usize) -> Result<usize, Error> {
        let neighbors = self
            .adjacency
            .get(&id)
            .ok_or(gamma::graph::Error::MissingNode(id))?;
        Ok(neighbors.len())
    }

    fn edges<'a>(&'a self) -> Box<dyn Iterator<Item = &'a (usize, usize)> + 'a> {
        Box::new(self.edges.iter())
    }

    fn has_edge(&self, sid: usize, tid: usize) -> Result<bool, Error> {
        Ok(self.contains_edge(sid, tid))
    }
}

impl AppendableGraph for DirtyGraph {
    fn add_node(&mut self) -> Result<usize, Error> {
        self.add_node_with(self.next_node)?;

        self.next_node += 1;

        Ok(self.next_node - 1)
    }

    fn add_node_with(&mut self, id: usize) -> Result<(), Error> {
        if self.nodes.contains(&id) {
            return Err(gamma::graph::Error::DuplicateNode(id));
        }

        self.nodes.insert(id);
        self.node_generation.insert(id, self.next_generation);
        self.adjacency.insert(id, vec![]);

        Ok(())
    }

    fn add_edge(&mut self, sid: usize, tid: usize) -> Result<(), Error> {
        self.edges.insert(new_edge(sid, tid));
        self.edge_generation
            .insert((sid, tid), self.next_generation);
        self.add_to_adjacency(sid, tid);
        self.add_to_adjacency(tid, sid);
        Ok(())
    }
}

impl RemovableGraph for DirtyGraph {
    fn remove_node(&mut self, id: usize) -> usize {
        self.remove_edges_with(id);
        self.node_generation.remove(&id);
        match self.nodes.remove(&id) {
            true => 1,
            false => 0,
        }
    }

    fn remove_edge(&mut self, sid: usize, tid: usize) -> usize {
        let edge = new_edge(sid, tid);
        match self.edges.remove(&edge) {
            true => 1,
            false => 0,
        }
    }

    fn remove_edges_with(&mut self, id: usize) -> usize {
        let to_remove: Vec<_> = self
            .edges
            .iter()
            .filter_map(|e| {
                if e.0 == id || e.1 == id {
                    Some(*e)
                } else {
                    None
                }
            })
            .collect();
        to_remove
            .into_iter()
            .map(|e| {
                // First, remove all generations
                self.edge_generation.remove(&e);
                // Then remove actual edges.
                match self.edges.remove(&e) {
                    true => 1,
                    false => 0,
                }
            })
            .sum()
    }
}
