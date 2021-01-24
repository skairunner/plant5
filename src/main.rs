mod logger;
mod rgg;

use crate::logger::start_logger;
use crate::rgg::{RggGraph, Rule};
use bevy::prelude::*;

/// The container for all the actual entities that form a plant.
struct Plant {
    pub rules: Vec<Rule>,
    pub graph: RggGraph,
}

/// Represents the corresponding visual part of a plant.
struct PlantNode {
    pub node_id: usize,
}

fn main() {
    start_logger();
    let rules: Vec<Rule> = serde_yaml::from_str(
        r#"
- from:
    nodes:
      - {id: 0, name: "growing tip"}
  to:
    - replace:
        target: 0
        with:
          name: "stem"
    - add:
        neighbors: [0]
        node:
          name: "growing tip"
    "#,
    )
    .unwrap();
    let mut plant = Plant {
        rules,
        graph: RggGraph::new(),
    };
    plant.graph.insert_node_with(crate::rgg::Node {
        name: "growing tip".to_string(),
        values: Default::default(),
    });
    plant.rules[0].apply(&mut plant.graph);
    plant.rules[0].apply(&mut plant.graph);
    log::error!("{:?}", plant.graph);
    App::build().add_plugins(DefaultPlugins).run();
}
