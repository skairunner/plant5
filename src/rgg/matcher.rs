use crate::rgg::rgg_graph::RggGraph;
use crate::rgg::rule::Rule;
use gamma::graph::{DefaultGraph, Graph};
use std::collections::HashMap;

impl Rule {
    pub fn matches<'a>(&'a self, graph: &'a RggGraph) -> MatchingState<'a> {
        MatchingState::new(&self, graph)
    }
}

pub enum MatchingDecision {
    NoMatch,
    Continue,
    Mapped,
}

/// Hold state for matching function.
pub struct MatchingState<'a> {
    graph: &'a RggGraph,
    rule: &'a Rule,
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

impl<'a> MatchingState<'a> {
    pub fn new(rule: &'a Rule, graph: &'a RggGraph) -> Self {
        let relations = rule.from.as_graph();

        Self {
            graph,
            rule,
            relations,
            mapping: HashMap::new(),
            pattern_index: 0,
            progress: maplit::hashmap! { 0 => 0 },
            graph_nodes: graph.graph.nodes().map(|e| *e).collect(),
        }
    }

    fn get_rule_id(&self, rules: &Rule) -> i32 {
        rules.from.nodes[self.pattern_index as usize].id
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
        if self.progress[&0] >= graph.graph.order() {
            log::info!("Reached end of graph. Exiting.");
            return MatchingDecision::NoMatch;
        }
        // Scan graph nodes until we find a match.
        let start = *self
            .progress
            .entry(self.pattern_index as usize)
            .or_insert(0);
        let end = self.graph_nodes.len();

        let rule_id = self.get_rule_id(rules);
        log::info!("Scanning for rule {}: {} to {}", rule_id, start, end);
        for i in start..end {
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

        // If we're on pattern index 0 and we're out of things, we're done here
        if self.pattern_index == 0 {
            return MatchingDecision::NoMatch;
        }

        // If failed to find next match, decrement the pattern_index, clearing out any bookmarks and mappings.
        self.mapping.remove(&rule_id);
        self.progress.remove(&(self.pattern_index as usize));
        self.pattern_index -= 1;

        MatchingDecision::Continue
    }

    /// For the current match, check that it satisfies the edges required.
    pub fn check_edges(&mut self, rules: &Rule) -> anyhow::Result<bool> {
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

    /// Once we verify a match, we can reset to find further matches by removing the mapping.
    pub fn reset_match(&mut self, rules: &Rule) {
        self.pattern_index -= 1;
        let r = self.mapping.remove(&self.get_rule_id(rules));
        if r.is_none() {
            log::info!("Tried to reset but there was nothing to reset.");
        }
    }
}

impl Iterator for MatchingState<'_> {
    type Item = HashMap<i32, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        #[allow(unused_assignments)]
        let mut output = HashMap::new();
        loop {
            let result = self.continue_search(self.rule, self.graph);
            match result {
                MatchingDecision::NoMatch => return None,
                MatchingDecision::Mapped => {
                    let result = self.check_edges(self.rule);
                    match &result {
                        Err(e) => panic!("{:?}", e),
                        Ok(r) => {
                            if *r {
                                output = self.mapping.clone();
                                self.reset_match(self.rule);
                                break;
                            } else {
                                self.reset_match(self.rule)
                            }
                        }
                    }
                }
                MatchingDecision::Continue => {}
            }
        }
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rgg::rule::{FromNode, NodeSet};
    use gamma::graph::AppendableGraph;
    use ntest::timeout;

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
        let to = vec![];
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
        let to = vec![];
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
    #[timeout(500)]
    fn test_match() {
        use maplit::hashmap;

        let rule = get_simple_test_rule();
        let graph = get_test_graph();
        let mut matcher = MatchingState::new(&rule, &graph);
        matcher.continue_search(&rule, &graph);
        assert_eq!(matcher.pattern_index, 1);
        assert!(matcher.check_edges(&rule).unwrap());
        assert_eq!(matcher.mapping, hashmap! { 0 => 0 });
    }

    #[test]
    #[timeout(500)]
    fn test_match_2() {
        use maplit::hashmap;
        use simplelog::*;

        TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();

        let rule = get_test_rule();
        let graph = get_test_graph();
        let mut matcher = rule.matches(&graph);
        let matched = matcher.next();

        match matched {
            None => panic!("No matches found"),
            Some(m) => {
                assert_eq!(matcher.pattern_index, 2);
                assert_eq!(m, hashmap! { 0 => 0, 1 => 1 });
            }
        }
    }

    #[test]
    #[timeout(500)]
    fn test_match_terminates() {
        simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default())
            .unwrap();
        let rule = get_test_rule();
        let graph = get_test_graph();
        let _v = rule.matches(&graph).collect::<Vec<_>>();
    }
}
