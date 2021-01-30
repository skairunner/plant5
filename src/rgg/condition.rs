use crate::rgg::Value;

/// Define a condition to match FromNodes against
#[derive(Debug, PartialEq, Clone)]
pub enum Condition {
    Equals(Value),
    LessThan(Value),
    GreaterThan(Value),
    LessThanOrEquals(Value),
    GreaterThanOrEquals(Value),
    /// Greater than Range.0, less than Range.1. Inclusive lower, exclusive upper.
    Range(Value, Value),
}
