use crate::shapes::{get_color, get_mesh};
use crate::{Plant, PlantNode};
use bevy::ecs::Entity;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::utils::{AHashExt, HashMap};
use gamma::graph::Graph;

/// Spawn a new node that corresponds to the provided node id.
pub fn spawn_node(
    node_id: usize,
    plant: &Plant,
    entities: &mut HashMap<(usize, usize), Entity>,
    offsets: &mut HashMap<(usize, usize), Vec3>,
    mesh_handles: &mut HashMap<(usize, usize), Handle<Mesh>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
) {
    // The key used to index into entities/offsets/meshes etc
    let ident = (plant.id, node_id);

    let mesh = get_mesh(&plant.graph.values[&node_id]);
    let mesh = meshes.add(mesh);
    mesh_handles.insert(ident, mesh.clone());
    let material = materials.add(StandardMaterial {
        albedo: get_color(plant.graph.values.get(&node_id).unwrap()),
        ..Default::default()
    });
    let parent_node = plant.graph.graph.get_ancestor(node_id);
    let (offset, parent) = if let Some(parent_node) = parent_node {
        let offset = offsets
            .get(&(plant.id, parent_node))
            .copied()
            .unwrap_or_else(|| {
                log::error!("Could not find PlantNode with id {}", parent_node);
                Vec3::default()
            });
        let parent = entities.get(&(plant.id, parent_node)).copied();
        (offset, parent)
    } else {
        (Vec3::zero(), None)
    };
    let rotation = match plant.graph.values.get(&node_id) {
        Some(node) => {
            if node.name == "shoot" {
                let degrees = node
                    .values
                    .get("rotation")
                    .map(|val| val.get::<f32>())
                    .unwrap_or_else(|| 0.0);
                Quat::from_rotation_x((45f32).to_radians())
                    * Quat::from_rotation_z(degrees.to_radians())
            } else {
                Quat::identity()
            }
        }
        None => Quat::identity(),
    };
    let plantnode = PlantNode {
        plant_id: plant.id,
        node_id,
        node_offset: Vec3::new(0.0, 0.0, 1.0),
    };
    // Need to insert the plantnode into our tracking dict
    offsets.insert(ident, plantnode.node_offset);

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

    entities.insert(ident, child);
    if let Some(parent) = parent {
        commands.push_children(parent, &[child]);
    } else {
        log::debug!("State: {:?}", plant.graph.graph);
        log::warn!("Could not find parent node for {:?}", ident)
    }
}

/// Creates all nodes of a plant.
pub fn spawn_plant_nodes(
    starting_node: usize,
    plant: &Plant,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
) {
    let mut entities = HashMap::new();
    let mut offsets = HashMap::new();
    let mut mesh_handles = HashMap::new();
    spawn_node(
        starting_node,
        plant,
        &mut entities,
        &mut offsets,
        &mut mesh_handles,
        meshes,
        materials,
        commands,
    );
    for node in plant.graph.graph.nodes().copied() {
        if node != starting_node {
            spawn_node(
                node,
                plant,
                &mut entities,
                &mut offsets,
                &mut mesh_handles,
                meshes,
                materials,
                commands,
            );
        }
    }
}
