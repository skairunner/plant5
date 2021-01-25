mod logger;
mod panorbit;
mod rgg;

use crate::logger::start_logger;
use crate::panorbit::{pan_orbit_camera, spawn_camera};
use crate::rgg::rule::RuleResult;
use crate::rgg::{RggGraph, Rule};
use bevy::ecs::bevy_utils::HashMap;
use bevy::prelude::*;
use bevy::utils::AHashExt;
use gamma::graph::Graph;
use std::collections::HashSet;

struct Tick(u64);

fn get_test_rules() -> Vec<Rule> {
    serde_yaml::from_str(
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
    .unwrap()
}

fn get_test_plant() -> Plant {
    let mut plant = Plant {
        rules: get_test_rules(),
        graph: RggGraph::new(),
    };
    plant.graph.insert_node_with(crate::rgg::Node {
        name: "growing tip".to_string(),
        values: Default::default(),
    });
    plant
}

/// The container for all the actual entities that form a plant.
struct Plant {
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
    pub node_id: usize,
    /// The coordinate to attach any children of this plant node
    pub node_offset: Vec3,
}

fn generate_stalk_mesh(width: f32, length: f32) -> Mesh {
    use bevy::render::mesh::shape::*;

    Box {
        min_x: -width / 2.0,
        max_x: width / 2.0,
        min_y: -width / 2.0,
        max_y: width / 2.0,
        min_z: 0.0,
        max_z: length,
    }
    .into()
}

fn update_plants(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tick: Res<Tick>,
    mut plant_query: Query<(&mut Plant,)>,
    node_query: Query<(&PlantNode, Entity)>,
) {
    let mut transforms = HashMap::new();
    let mut offsets = HashMap::new();
    for (node, entity) in node_query.iter() {
        transforms.insert(node.node_id, entity);
        offsets.insert(node.node_id, node.node_offset);
    }

    for (mut plant,) in plant_query.iter_mut() {
        let mut added_nodes = HashSet::new();
        if tick.0 > 3 && plant.graph.graph.order() < 3 {
            let results = plant.do_rules();
            added_nodes.extend(results.added.into_iter());
        }
        for id in added_nodes {
            let mesh = generate_stalk_mesh(0.1, 4.0);
            let mesh = meshes.add(mesh);
            let material = materials.add(StandardMaterial {
                albedo: Color::rgb(0.5, 0.5, 0.5),
                ..Default::default()
            });
            let parent_node = *plant.graph.graph.neighbors(id).unwrap().next().unwrap();
            let offset = offsets.get(&parent_node).map(|v| *v).unwrap_or_else(|| {
                log::error!("Could not find PlantNode with id {}", parent_node);
                Vec3::default()
            });
            let parent = transforms.get(&parent_node).map(|e| *e).expect(&format!(
                "An entity corresponding to the node id {}",
                parent_node
            ));
            commands.set_current_entity(parent);
            commands.with_children(|p| {
                log::info!("{} was added", id);
                p.spawn((PlantNode {
                    node_id: id,
                    node_offset: Vec3::new(0.0, 0.0, 4.0),
                },))
                    .with_bundle(PbrBundle {
                        mesh,
                        material,
                        transform: Transform {
                            translation: offset,
                            rotation: Quat::from_rotation_y(0.52),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
            });
        }
    }
}

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = generate_stalk_mesh(0.1, 4.0);
    let mesh = meshes.add(mesh);
    let material = materials.add(StandardMaterial::default());
    commands
        .spawn((get_test_plant(),))
        .spawn((PlantNode {
            node_id: 0,
            node_offset: Vec3::new(0.0, 0.0, 4.0),
        },))
        .with_bundle(PbrBundle {
            mesh,
            material,
            transform: Transform {
                translation: Vec3::default(),
                ..Default::default()
            },
            ..Default::default()
        })
        .spawn(LightBundle {
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
