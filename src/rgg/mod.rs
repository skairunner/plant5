// Module for Relational Growth Grammars
// Not a full-fledged RGG (yet?) because it's devilishly difficult, but it still acts on graphs

mod dirty_graph;
pub mod matcher;
pub mod node;
mod procedures;
mod rgg_graph;
mod rule;
mod serde;

pub use node::{Node, Value};
