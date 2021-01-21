// Module for Relational Growth Grammars
// Not a full-fledged RGG (yet?) because it's devilishly difficult, but it still acts on graphs

pub mod condition;
mod dirty_graph;
pub mod matcher;
pub mod node;
mod procedures;
mod rgg_graph;
pub mod rule;
mod serde;

pub use condition::Condition;
pub use node::{Node, Value};
pub use rule::{FromNode, NodeSet, Rule};
