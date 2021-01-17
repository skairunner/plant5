use std::collections::HashMap;

#[derive(Clone, Debug)]
/// Represents the values stored in a node in an RGG.
pub struct Node {
    pub name: String,
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

#[derive(Debug, Clone)]
pub enum RGGType {
    Int,
    Float,
}

#[derive(Clone, Debug)]
pub struct Value {
    raw_value: *mut i32,
    rgg_type: RGGType,
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            self.raw_value.drop_in_place();
        }
    }
}

impl Value {
    pub fn new(value_type: RGGType) -> Self {
        let pointer = match value_type {
            RGGType::Int => Box::into_raw(Box::new(0)),
            RGGType::Float => Box::into_raw(Box::new(0f32)) as *mut i32,
        };
        Self {
            raw_value: pointer,
            rgg_type: value_type,
        }
    }

    pub fn get<T: Copy>(&self) -> T {
        unsafe { *(self.raw_value as *mut T) }
    }

    pub fn get_mut<T: Copy>(&mut self) -> &mut T {
        unsafe { &mut *(self.raw_value as *mut T) }
    }

    pub fn set_f32(&mut self, mut v: f32) {
        unsafe {
            self.raw_value.drop_in_place();
            self.raw_value = &mut v as *mut f32 as *mut i32;
        }
    }

    pub fn set_i32(&mut self, mut v: i32) {
        unsafe {
            self.raw_value.drop_in_place();
            self.raw_value = &mut v as *mut i32;
        }
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
