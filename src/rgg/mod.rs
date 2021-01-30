// Module for Relational Growth Grammars
// Not a full-fledged RGG (yet?) because it's devilishly difficult, but it still acts on graphs

pub mod condition;
mod dirty_graph;
pub mod matcher;
pub mod node;
mod procedures;
pub mod rgg_graph;
pub mod rule;
mod serde;
pub mod value;

pub use condition::Condition;
pub use node::{FromNode, Node};
pub use rgg_graph::RggGraph;
pub use rule::{NodeSet, Rule};
pub use value::Value;
