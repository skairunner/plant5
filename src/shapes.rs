// Include primitives for rendering.
use crate::rgg::Node;
use bevy::render::mesh::{
    shape::{Box, Icosphere},
    Mesh,
};

pub fn get_mesh(node: &Node) -> Mesh {
    match node.name.as_str() {
        "stem" => {
            let len = node
                .values
                .get("len")
                .map(|v| v.get::<f32>())
                .unwrap_or_else(|| {
                    log::warn!("Missing len parameter");
                    1.0
                });
            stalk(0.1, len)
        }
        n => {
            log::error!("Could not find mesh for {:?}", n);
            Icosphere {
                radius: 0.2,
                subdivisions: 2,
            }
            .into()
        }
    }
}

pub fn stalk(width: f32, length: f32) -> Mesh {
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
