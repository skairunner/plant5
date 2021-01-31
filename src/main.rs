mod logger;
mod panorbit;
mod rgg;
mod shapes;

use crate::logger::start_logger;
use crate::panorbit::{pan_orbit_camera, spawn_camera};
use crate::rgg::rule::RuleResult;
use crate::rgg::{RggGraph, Rule};
use crate::shapes::{get_mesh, stalk};
use bevy::ecs::bevy_utils::HashMap;
use bevy::prelude::*;
use bevy::utils::AHashExt;

struct Tick(u64);

fn get_test_rules() -> Vec<Rule> {
    serde_yaml::from_str(
        r#"
# Split stem into two
- from:
    nodes:
        - {id: 0, name: "stem"}
  to:
    - add:
        neighbors: [0]
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
    plant
}

/// The container for all the actual entities that form a plant.
struct Plant {
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
                let mesh = get_mesh(&plant.graph.values[&id]);
                let mesh = meshes.add(mesh);
                mesh_handles.insert((plant.id, id), mesh.clone());
                let material = materials.add(StandardMaterial {
                    albedo: Color::rgb(0.5, 0.5, 0.5),
                    ..Default::default()
                });
                let parent_node = plant.graph.graph.get_ancestor(id);
                let (offset, parent) = if let Some(parent_node) = parent_node {
                    let offset = offsets
                        .get(&(plant.id, parent_node))
                        .copied()
                        .unwrap_or_else(|| {
                            log::error!("Could not find PlantNode with id {}", parent_node);
                            Vec3::default()
                        });
                    let parent = entities
                        .get(&(plant.id, parent_node))
                        .copied()
                        .unwrap_or_else(|| {
                            panic!("An entity corresponding to the node id {}. Graph {:?}", parent_node, plant.graph)
                        });
                    (offset, Some(parent))
                } else {
                    (Vec3::zero(), None)
                };
                let rotation = match plant.graph.values.get(&id) {
                    Some(node) => {
                        if node.name == "shoot" {
                            let degrees = node.values.get("rotation")
                                .map(|val| val.get::<f32>())
                                .unwrap_or_else(|| 0.0);
                            Quat::from_rotation_x((45f32).to_radians()) * Quat::from_rotation_z(degrees.to_radians())
                        } else {
                            Quat::identity()
                        }
                    }
                    None => Quat::identity(),
                };
                let plantnode = PlantNode {
                    plant_id: plant.id,
                    node_id: id,
                    node_offset: Vec3::new(0.0, 0.0, 1.0),
                };
                // Need to insert the plantnode into our tracking dict
                offsets.insert((plant.id, id), plantnode.node_offset);

                let child = commands
                    .spawn((plantnode,))
                    .with_bundle(PbrBundle {
                        mesh,
                        material,
                        transform: Transform {
                            translation: offset,
                            rotation,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .current_entity()
                    .expect("that we just spawned an entity");

                entities.insert((plant.id, id), child);
                if let Some(parent) = parent {
                    commands.push_children(parent, &[child]);
                }
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
    let mesh = stalk(0.1, 4.0);
    let mesh = meshes.add(mesh);
    let material = materials.add(StandardMaterial::default());
    commands
        .spawn((get_test_plant(0),))
        .spawn((PlantNode {
            plant_id: 0,
            node_id: 0,
            node_offset: Vec3::new(0.0, 0.0, 1.0),
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
