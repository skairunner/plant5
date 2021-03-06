use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum RGGType {
    Int,
    Float,
}

#[derive(Clone)]
pub struct Value {
    pub(super) raw_value: i32,
    pub(super) rgg_type: RGGType,
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Value")
            .field("raw_value", &self.raw_value)
            .field("as float", &self.get::<f32>())
            .field("rgg_type", &self.rgg_type)
            .finish()
    }
}

impl Value {
    pub fn new(value_type: RGGType) -> Self {
        let pointer = match value_type {
            RGGType::Int => Box::into_raw(Box::new(0)),
            RGGType::Float => Box::into_raw(Box::new(0f32)) as *mut i32,
        };
        Self {
            raw_value: unsafe { *pointer },
            rgg_type: value_type,
        }
    }

    pub fn new_int(i: i32) -> Self {
        Self {
            raw_value: i,
            rgg_type: RGGType::Int,
        }
    }

    pub fn new_float(f: f32) -> Self {
        let p = Box::into_raw(Box::new(f)) as *mut i32;
        Self {
            raw_value: unsafe { *p },
            rgg_type: RGGType::Float,
        }
    }

    pub fn get<T: Copy>(&self) -> T {
        unsafe { *(&self.raw_value as *const i32 as *const T) }
    }

    pub fn get_mut<T: Copy>(&mut self) -> &mut T {
        unsafe { &mut *(self.raw_value as *mut T) }
    }

    pub fn set_f32(&mut self, v: f32) {
        let p = Box::into_raw(Box::new(v)) as *const i32;
        unsafe {
            self.raw_value = *p;
        }
    }

    pub fn set_i32(&mut self, v: i32) {
        self.raw_value = v;
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.rgg_type == other.rgg_type && self.raw_value == other.raw_value
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::new_float(f)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::new_int(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_i32() {
        let v = Value::new(RGGType::Int);
        assert_eq!(v.get::<i32>(), 0);
    }

    #[test]
    fn test_get_f32() {
        let v = Value::new(RGGType::Float);
        assert_eq!(v.get::<f32>(), 0f32);
    }

    #[test]
    fn test_set_i32() {
        let mut v = Value::new(RGGType::Int);
        v.set_i32(3142);
        assert_eq!(v.get::<i32>(), 3142);
    }

    #[test]
    fn test_set_f32() {
        let mut v = Value::new(RGGType::Float);
        v.set_f32(3142.1);
        assert_eq!(v.get::<f32>(), 3142.1);
    }
}
