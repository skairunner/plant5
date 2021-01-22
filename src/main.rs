mod rgg;

use crate::rgg::{RggGraph, Rule};
use bevy::prelude::*;

struct Plant {
    pub rules: Vec<Rule>,
    pub graph: RggGraph,
}

fn main() {
    use simplelog::*;
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();
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
    // plant.rules[0].apply(&mut plant.graph);
    println!("{:?}", plant.graph);
    App::build().add_plugins(DefaultPlugins).run();
}
