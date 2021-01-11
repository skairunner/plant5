// Module for Relational Growth Grammars
// Not a full-fledged RGG (yet?) because it's devilishly difficult, but it still acts on graphs

mod dirty_graph;
pub mod matcher;
pub mod node;
mod rgg_graph;
mod rule;

pub use node::{Node, Value};
