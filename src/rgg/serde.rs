use crate::rgg::node::RGGType;
use crate::rgg::procedures::*;
use crate::rgg::{Node, Value};
use serde::de::{Error, Expected, Unexpected};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
struct RawValue(String);

#[derive(Deserialize)]
struct I32(i32);

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = RawValue::deserialize(deserializer)?.0;
        match s.parse::<i32>() {
            Ok(i) => Ok(Value::from(i)),
            Err(_) => {
                let f = s
                    .parse::<f32>()
                    .map_err(|e| D::Error::invalid_type(Unexpected::Str(&s), &"float or int"))?;
                Ok(Value::from(f))
            }
        }
    }
}

impl<'de> Deserialize<'de> for DeleteProcedure {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let i = I32::deserialize(deserializer)?;
        Ok(DeleteProcedure { target: i.0 })
    }
}

#[cfg(test)]
mod test {
    use super::super::procedures::*;
    use crate::rgg::{Node, Value};

    #[test]
    fn test_de_value_float() {
        let val: Value = serde_yaml::from_str("0.3").unwrap();
        assert_eq!(val.get::<f32>(), 0.3);
    }

    #[test]
    fn test_de_value_int() {
        let val: Value = serde_yaml::from_str("3").unwrap();
        assert_eq!(val.get::<i32>(), 3);
    }

    #[test]
    fn test_de_node() {
        let node: Node = serde_yaml::from_str(
            r#"
name: "hi"
values:
  foo: 1
  bar: 2.0"#,
        )
        .unwrap();

        assert_eq!(node.name, "hi".to_string());
        assert_eq!(node.values["foo"].get::<i32>(), 1);
        assert_eq!(node.values["bar"].get::<f32>(), 2.0);
    }

    #[test]
    fn test_de_delete_proc() {
        let proc: DeleteProcedure = serde_yaml::from_str("3").unwrap();
        assert_eq!(proc.target, 3);
    }

    #[test]
    fn test_de_delete() {
        let proc: Procedure = serde_yaml::from_str("delete: 49").unwrap();
        match &proc {
            Procedure::Delete(proc) => assert_eq!(proc.target, 49),
            _ => panic!("Invalid procedure: {:?}", proc),
        }
    }

    #[test]
    fn test_de_add() {
        let proc: Procedure = serde_yaml::from_str(
            r#"
add:
  node:
    name: "hi"
    values:
      foo: 1
      bar: 2.0
  neighbors: [1, 2, 3, 4]"#,
        )
        .unwrap();
        match &proc {
            Procedure::Add(proc) => {
                assert_eq!(proc.new_node.name, "hi");
                assert_eq!(proc.new_node.values["foo"].get::<i32>(), 1);
                assert_eq!(proc.neighbors, vec![1, 2, 3, 4]);
            }
            _ => panic!("Invalid procedure: {:?}", proc),
        }
    }

    #[test]
    fn test_de_replace() {
        let proc: Procedure = serde_yaml::from_str(
            r#"
replace:
  target: 0
  replace:
    name: "hi"
    values:
      foo: 5.5
      bar: 2"#,
        )
        .unwrap();
        match &proc {
            Procedure::Replace(proc) => {
                assert_eq!(proc.replacement.name, "hi");
                assert_eq!(proc.replacement.values["foo"].get::<f32>(), 5.5);
                assert_eq!(proc.replacement.values["bar"].get::<i32>(), 2);
                assert_eq!(proc.target, 0);
            }
            _ => panic!("Invalid procedure: {:?}", proc),
        }
    }

    #[test]
    fn test_de_merge() {
        let proc: Procedure = serde_yaml::from_str(
            r#"
replace:
  target: 0
  replace:
    name: "hi"
    values:
      foo: 5.5
      bar: 2"#,
        )
        .unwrap();
        match &proc {
            Procedure::Merge(proc) => {}
            _ => panic!(),
        }
    }
}
