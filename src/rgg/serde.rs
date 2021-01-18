use crate::rgg::node::RGGType;
use crate::rgg::procedures::*;
use crate::rgg::rule::Condition;
use crate::rgg::{Node, Value};
use core::fmt::Formatter;
use serde::de::{Error, Expected, SeqAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
struct RawValue(String);

#[derive(Deserialize)]
struct I32(i32);

#[derive(Deserialize)]
struct MyVec(Vec<i32>);

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

impl<'de> Deserialize<'de> for MergeProcedure {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let mut vec = MyVec::deserialize(deserializer)?.0;
        if vec.len() < 2 {
            return Err(Error::invalid_length(vec.len(), &"at least 2"));
        }

        let final_node = vec.remove(0);
        Ok(Self {
            targets: vec,
            final_node,
        })
    }
}

impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ConditionVisitor)
    }
}

struct ConditionVisitor;

impl ConditionVisitor {
    fn parse_condition(designator: String, value: Value) -> Condition {
        match designator.as_str() {
            "eq" => Condition::Equals(value),
            "lt" => Condition::LessThan(value),
            "gt" => Condition::GreaterThan(value),
            "lte" => Condition::LessThanOrEquals(value),
            "gte" => Condition::GreaterThanOrEquals(value),
            e => panic!("Invalid designator {:?} should have been caught earlier", e),
        }
    }
}

impl<'de> Visitor<'de> for ConditionVisitor {
    type Value = Condition;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a sequence of a string followed by one or two values, representing a Condition",
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error>
    where
        A: SeqAccess<'de>,
    {
        let designation: String = seq
            .next_element()?
            .ok_or_else(|| <A as SeqAccess<'de>>::Error::custom("Missing designator"))?;
        match designation.as_str() {
            "eq" | "lt" | "gt" | "lte" | "gte" => {
                let value: Value = seq
                    .next_element()?
                    .ok_or_else(|| <A as SeqAccess<'de>>::Error::custom("Missing value"))?;
                Ok(Self::parse_condition(designation, value))
            }
            "range" => {
                let range_low: Value = seq.next_element()?.ok_or_else(|| {
                    <A as SeqAccess<'de>>::Error::custom("Missing value for range lower")
                })?;
                let range_high: Value = seq.next_element()?.ok_or_else(|| {
                    <A as SeqAccess<'de>>::Error::custom("Missing value for range upper")
                })?;
                Ok(Condition::Range(range_low, range_high))
            }
            e => Err(<A as SeqAccess<'de>>::Error::custom(format!(
                "Unknown designator {:?}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::procedures::*;
    use crate::rgg::rule::Condition;
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
        let proc: Procedure = serde_yaml::from_str("merge: [3, 1, 2, 3]").unwrap();
        match &proc {
            Procedure::Merge(proc) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn test_de_conditions() {
        let conditions: Vec<Condition> = serde_yaml::from_str(
            r#"
- [eq, 3]
- [lt, 2.0]
- [gt, 3.0]
- [lte, 10]
- [gte, -3]
- [range, 0, 2]"#,
        )
        .unwrap();
    }
}
