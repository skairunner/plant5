use std::collections::HashMap;

use super::Value;
use crate::rgg::Condition;
use meval::Context;
use rand::Rng;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// Represents the values stored in a node in an RGG.
pub struct Node {
    pub name: String,
    #[serde(default)]
    pub values: HashMap<String, Value>,
}

impl Node {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: Default::default(),
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new("")
    }
}

/// Identify a node to match against
#[derive(Deserialize)]
pub struct FromNode {
    /// Identify the node in the context of a rule
    pub id: i32,
    /// Identify the "name" of the node. Optional.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    /// Specify any potential values the node has.
    pub values: HashMap<String, Condition>,
}

impl FromNode {
    /// Check whether the node can match the provided node.
    pub fn match_node(&self, node: &Node) -> bool {
        // If name is specified, needs to match.
        if let Some(name) = self.name.as_ref() {
            if *name != node.name {
                return false;
            }
        }

        // If any values are specified, need to match conditions.
        // TODO

        true
    }
}

/// Define a replacement node.
/// For replace, can use operations relative to the previous node's values.
/// For all nodes, can use some operations for values, such as rand
#[derive(Deserialize, Debug, Clone)]
pub struct ToNode {
    pub name: String,
    pub values: HashMap<String, String>,
}

impl ToNode {
    /// Evaluate the values of the tonode to create a normal node
    pub fn eval(&self, base_node: Option<&Node>) -> Node {
        let mut context = Context::new();
        if let Some(base_node) = base_node {
            for (name, value) in &base_node.values {
                context.var(name, value.get::<f32>() as f64);
            }
        }
        context.func2("rand", |min, max| {
            let mut rng = rand::thread_rng();
            rng.gen_range(min..max)
        });
        let mut values = HashMap::new();
        for (name, expr) in &self.values {
            let val = meval::eval_str_with_context(expr, &context).unwrap();
            values.insert(name.to_string(), Value::new_float(val as f32));
        }

        Node {
            name: self.name.clone(),
            values,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_tonode_simple() {
        let context = Node {
            name: "Hi".to_string(),
            values: serde_yaml::from_str(
                r#"
age: 30.0
            "#,
            )
            .unwrap(),
        };
        let tonode = ToNode {
            name: "bye".to_string(),
            values: maplit::hashmap! {
                "age".to_string() => "age - 1".to_string()
            },
        };
        let result = tonode.eval(Some(&context));
        assert_eq!(result.name, "bye");
        assert_eq!(result.values["age"].get::<f32>(), 29.0);
    }

    #[test]
    fn test_tonode_method() {
        let tonode = ToNode {
            name: "bye".to_string(),
            values: maplit::hashmap! {
                "len".to_string() => "rand(1, 5)".to_string()
            },
        };
        let result = tonode.eval(None);
        assert_eq!(result.name, "bye");
        let len = result.values["len"].get::<f32>();
        assert!(len > 0.0);
        assert!(len < 5.0);
    }
}
