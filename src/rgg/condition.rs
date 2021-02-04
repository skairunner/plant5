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

impl Condition {
    /// Check whether the provided value fulfils the condition
    pub fn check<T: PartialEq + Copy + PartialOrd>(&self, value: T) -> bool {
        match self {
            Self::Equals(condition) => value == condition.get::<T>(),
            Self::LessThan(condition) => value < condition.get::<T>(),
            Self::GreaterThan(condition) => value > condition.get::<T>(),
            Self::LessThanOrEquals(condition) => value <= condition.get::<T>(),
            Self::GreaterThanOrEquals(condition) => value >= condition.get::<T>(),
            Self::Range(l, r) => l.get::<T>() <= value && value <= r.get::<T>(),
        }
    }
}
