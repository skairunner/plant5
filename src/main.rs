mod rgg;

use crate::rgg::Rule;
use bevy::prelude::*;

fn main() {
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
    App::build().add_plugins(DefaultPlugins).run();
}
