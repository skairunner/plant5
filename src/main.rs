mod logger;
mod panorbit;
mod plant;
mod rgg;
mod shapes;

use crate::logger::start_logger;
use crate::panorbit::{pan_orbit_camera, spawn_camera};
use crate::plant::{spawn_node, spawn_plant_nodes};
use crate::rgg::rule::RuleResult;
use crate::rgg::{RggGraph, Rule};
use crate::shapes::{get_color, get_mesh, stalk};
use bevy::ecs::bevy_utils::HashMap;
use bevy::prelude::*;
use bevy::utils::AHashExt;
use gamma::graph::AppendableGraph;

struct Tick(u64);

fn get_test_rules() -> Vec<Rule> {
    serde_yaml::from_str(
        r#"
# Split a 2stem into a 3stem
- from:
    nodes:
        - {id: 0, name: "stem"}
        - {id: 1, name: "stem"}
    edges:
        - [0, 1]
  to:
    - add:
        neighbors: [1]
        node:
          name: "stem"
          values:
            dir: dir + 1
# Create a sideshoot if it doesn't already have one
- from:
    nodes:
      - id: 0
        name: "stem"
        values:
          sprouted: [eq, 0]
  to:
    - replace:
        target: 0
        with:
          name: "stem"
          values:
            sprouted: 1
            dir: dir
    - add:
        neighbors: [0]
        node:
          name: "shoot"
          values:
            rotation: 90 * dir
    "#,
    )
    .unwrap()
}

fn get_test_plant(id: usize) -> Plant {
    let mut plant = Plant {
        id,
        rules: get_test_rules(),
        graph: RggGraph::new(),
    };
    plant.graph.insert_node_with(crate::rgg::Node {
        name: "stem".to_string(),
        values: serde_yaml::from_str("{dir: 0, sprouted: 0}").unwrap(),
    });
    plant.graph.insert_node_with(crate::rgg::Node {
        name: "stem".to_string(),
        values: serde_yaml::from_str("{dir: 0, sprouted: 0}").unwrap(),
    });
    plant.graph.graph.add_edge(0, 1).unwrap();
    plant
}

/// The container for all the actual entities that form a plant.
pub struct Plant {
    pub id: usize,
    pub rules: Vec<Rule>,
    pub graph: RggGraph,
}

impl Plant {
    pub fn do_rules(&mut self) -> RuleResult {
        let mut result = RuleResult::new();
        for rule in &self.rules {
            result.add(rule.apply(&mut self.graph));
        }

        result
    }
}

/// Represents the corresponding visual part of a plant.
struct PlantNode {
    pub plant_id: usize,
    pub node_id: usize,
    /// The coordinate to attach any children of this plant node
    pub node_offset: Vec3,
}

fn update_plants(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tick: Res<Tick>,
    mut plant_query: Query<(&mut Plant,)>,
    node_query: Query<(&PlantNode, Entity, &Handle<Mesh>)>,
    mut offset_query: Query<(&PlantNode, &mut Transform)>,
) {
    let mut entities = HashMap::new();
    let mut offsets = HashMap::new();
    let mut mesh_handles = HashMap::new();
    // store offsets that need editing
    let mut edit_offsets = HashMap::new();
    for (node, entity, mesh) in node_query.iter() {
        entities.insert((node.plant_id, node.node_id), entity);
        offsets.insert((node.plant_id, node.node_id), node.node_offset);
        mesh_handles.insert((node.plant_id, node.node_id), (*mesh).clone());
    }

    for (mut plant,) in plant_query.iter_mut() {
        if tick.0 > 3 && plant.graph.order() < 5 {
            let results = plant.do_rules();
            log::info!("Rule results for plant {}: {:?}", plant.id, results);
            // Handle added
            for id in results.added {
                spawn_node(
                    id,
                    &plant,
                    &mut entities,
                    &mut offsets,
                    &mut mesh_handles,
                    &mut meshes,
                    &mut materials,
                    commands,
                );
            }
            // Handle modified
            for id in results.modified {
                let handle = &mesh_handles[&(plant.id, id)];
                let node = &plant.graph.values[&id];
                meshes.set(handle, get_mesh(node));

                let offset = offsets[&(plant.id, id)];

                // Also queue offsets for children
                for child in plant.graph.graph.get_children(id) {
                    edit_offsets.insert((plant.id, child), offset);
                }
            }

            // End result
            log::info!(
                "Results: {}\n    {:?}",
                plant.graph.as_dot_string(),
                plant.graph.values
            );
        }
    }

    // Actually do the offsets queued
    for (node, mut transform) in offset_query.iter_mut() {
        if let Some(offset) = edit_offsets.get(&(node.plant_id, node.node_id)) {
            transform.translation = *offset;
        }
    }
}

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let plant = get_test_plant(0);
    spawn_plant_nodes(0, &plant, &mut meshes, &mut materials, commands);
    commands.spawn((plant,)).spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}

fn do_tick(mut tick: ResMut<Tick>) {
    tick.0 += 1;
}

fn main() {
    start_logger();

    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn_camera.system())
        .add_startup_system(setup.system())
        .add_system(do_tick.system())
        .add_system(pan_orbit_camera.system())
        .add_system(update_plants.system())
        .add_resource(Tick(0))
        .run();
}
