use crate::rgg::rgg_graph::RggGraph;
use crate::rgg::{Node, Value};
use gamma::graph::{AppendableGraph, DefaultGraph, Graph};
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

impl Rule {
    /// Attempt to apply the rule to the graph as many times as possible.
    /// When applying rules, we don't match the "from" against dirty nodes/edges,
    /// i.e. nodes/edges that have already been replaced by a rule.
    pub fn apply(&self, graph: &mut RggGraph) {}

    /// Match the From rules against the graph.
    fn do_match(&self, graph: &RggGraph) {
        // Build a simple graph that contains the edge relations of the ruleset.
        let relations = self.from.as_graph();

        let mut state = MatchingState::new(self, graph);

        // Map from rule index to graph id
        let mut node_mapping = HashMap::new();
        // The ruleset node index
        let mut node_index_to_match = 0;
        // For now, match purely structure.
        for node in graph.graph.nodes() {
            if self.from.nodes[node_index_to_match].match_node(&graph.values[node]) {
                node_mapping.insert(node_index_to_match, *node);
                node_index_to_match += 1;
            }
        }
    }

    /// Grow the set of rulenodes that were matched to graphnodes, checking against relations as well as the node itself.
    fn add_node(&self, graph: &RggGraph) {}
}

enum MatchingDecision {
    NoMatch,
    Continue,
    Mapped,
}

/// Hold state for matching function.
struct MatchingState {
    /// The necessary relations (between rule nodes) for a match.
    /// Uses the rule node id.
    relations: DefaultGraph,
    /// Discovered mappings between rule node ids and Rgg ids.
    mapping: HashMap<i32, usize>,
    /// The current pattern node we are checking against.
    pattern_index: i32,
    /// Store the last node up to which we have checked for a given rule node.
    progress: HashMap<usize, usize>,
    /// Indexable copy of the graph nodes
    graph_nodes: Vec<usize>,
}

impl MatchingState {
    pub fn new(rule: &Rule, graph: &RggGraph) -> Self {
        let relations = rule.from.as_graph();

        Self {
            relations,
            mapping: HashMap::new(),
            pattern_index: 0,
            progress: HashMap::new(),
            graph_nodes: graph.graph.nodes().map(|e| *e).collect(),
        }
    }

    /// Find the next tentative match (disregarding edge relations)
    pub fn continue_search(&mut self, rules: &Rule, graph: &RggGraph) -> MatchingDecision {
        log::info!("Continuing search...");
        // Exit condition 1: We have found a match that needs to be handled.
        if rules.from.nodes.len() <= self.pattern_index as usize {
            log::info!("All {} rules were matched.", rules.from.nodes.len());
            return MatchingDecision::Mapped;
        }
        // Exit condition 2: There are no matches at all
        if self.pattern_index <= -1 {
            log::info!("Patterns is -1. Quitting.");
            return MatchingDecision::NoMatch;
        }
        // Scan graph nodes until we find a match.
        let start = self
            .progress
            .entry(self.pattern_index as usize)
            .or_insert(0);
        let end = self.graph_nodes.len();

        let rule_id = rules.from.nodes[self.pattern_index as usize].id;
        log::info!("Scanning for rule {}: {} to {}", rule_id, *start, end);
        for i in *start..end {
            // Skip if already matched.
            if self.mapping.contains_key(&rule_id) {
                log::info!("Skipping rule {} because it was already matched", rule_id);
                continue;
            }
            if rules.from.nodes[self.pattern_index as usize].match_node(&graph.values[&i]) {
                // Add it as a tentative match
                log::info!(
                    "Inserting tentative match {}->{}",
                    rule_id,
                    self.graph_nodes[i]
                );
                self.mapping.insert(rule_id, self.graph_nodes[i]);
                // Bookmark progress
                self.progress.insert(self.pattern_index as usize, i + 1);
                log::info!("Bookmarked {}->{}", self.pattern_index, i + 1);
                // Look for next rule.
                self.pattern_index += 1;
                return MatchingDecision::Continue;
            }
        }

        // If failed to find next match, decrement the pattern_index, clearing out any bookmarks and mappings
        self.mapping.remove(&rule_id);
        self.progress.remove(&(self.pattern_index as usize));
        self.pattern_index -= 1;

        MatchingDecision::Continue
    }

    /// For the current match, check that it satisfies the edges required.
    pub fn check_edges(&mut self, rules: &Rule, rgg: &RggGraph) -> anyhow::Result<bool> {
        if (self.pattern_index as usize) < rules.from.nodes.len() {
            panic!("Didn't map all the nodes");
        }

        for edge in rules.from.edges.iter() {
            // Look up the from/to that we matched to.
            let from = &self.mapping[&edge.0];
            let to = &self.mapping[&edge.1];
            if !self.relations.has_edge(*from, *to)? {
                log::info!("({}, {}) not in graph {:?}", edge.0, edge.1, self.relations);
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Once we verify a match, we can reset to find further matches
    pub fn reset_match(&mut self) {
        self.pattern_index -= 1;
        let r = self.mapping.remove(&self.pattern_index);
        if r.is_none() {
            log::info!("Tried to reset but there was nothing to reset.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This rule takes a generic node, then adds a new node.
    fn get_simple_test_rule() -> Rule {
        let from = NodeSet {
            nodes: vec![FromNode {
                id: 0,
                name: None,
                values: Default::default(),
            }],
            edges: vec![],
        };
        let to = NodeSet {
            nodes: vec![
                ToNode {
                    id: 0,
                    name: None,
                    values: Default::default(),
                },
                ToNode {
                    id: 1,
                    name: None,
                    values: Default::default(),
                },
            ],
            edges: vec![(0, 1)],
        };
        Rule { from, to }
    }

    /// This rule takes two generic nodes connected by an edge, and adds a third node.
    fn get_test_rule() -> Rule {
        let from = NodeSet {
            nodes: vec![
                FromNode {
                    id: 0,
                    name: None,
                    values: Default::default(),
                },
                FromNode {
                    id: 1,
                    name: None,
                    values: Default::default(),
                },
            ],
            edges: vec![(0, 1)],
        };
        let to = NodeSet {
            nodes: vec![
                ToNode {
                    id: 0,
                    name: None,
                    values: Default::default(),
                },
                ToNode {
                    id: 1,
                    name: None,
                    values: Default::default(),
                },
            ],
            edges: vec![(0, 1)],
        };
        Rule { from, to }
    }

    fn get_test_graph() -> RggGraph {
        let mut graph = RggGraph::new();
        graph.insert_node();
        graph.insert_node();
        graph.graph.add_edge(0, 1).unwrap();
        graph
    }

    #[test]
    fn test_match() {
        use maplit::hashmap;

        let rule = get_simple_test_rule();
        let graph = get_test_graph();
        let mut matcher = MatchingState::new(&rule, &graph);
        matcher.continue_search(&rule, &graph);
        assert_eq!(matcher.pattern_index, 1);
        assert!(matcher.check_edges(&rule, &graph).unwrap());
        assert_eq!(matcher.mapping, hashmap! { 0 => 0 });
    }

    #[test]
    fn test_match_2() {
        use maplit::hashmap;
        use simplelog::*;

        TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed);

        let rule = get_test_rule();
        let graph = get_test_graph();
        let mut matcher = MatchingState::new(&rule, &graph);

        loop {
            let result = matcher.continue_search(&rule, &graph);
            match result {
                MatchingDecision::NoMatch => break,
                MatchingDecision::Mapped => {
                    let result = matcher.check_edges(&rule, &graph);
                    match &result {
                        Err(e) => panic!("{:?}", e),
                        Ok(r) => {
                            if *r {
                                break;
                            } else {
                                log::info!("Edges did not match for mapping {:?}", matcher.mapping);
                                matcher.reset_match()
                            }
                        }
                    }
                }
                MatchingDecision::Continue => {}
            }
        }
        assert_eq!(matcher.pattern_index, 2);
        assert_eq!(matcher.mapping, hashmap! { 0 => 0, 1 => 1 });
    }
}
