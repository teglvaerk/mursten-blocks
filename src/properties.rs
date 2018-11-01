use std::slice::{Iter, IterMut};

pub trait GetProperties {
    fn properties<'a>(&'a mut self) -> Properties;
}

pub struct Properties<'a> {
    properties: Vec<Box<Property<'a> + 'a>>,
}

pub trait Property<'a> {
    fn name(&self) -> &'static str;
    fn set(&mut self, value: Value);
    fn get(&self) -> Value;
}

#[derive(Debug)]
pub enum Value {
    Float(f32),
    Integer(i32),
    Bool(bool),
}

impl<'a> Properties<'a> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
    pub fn add<T>(mut self, name: &'static str, reference: &'a mut T) -> Self
    where
        T: Clone + From<Value> + Into<Value>,
    {
        self.properties.retain(|p| p.name() != name);
        let property_reference = PropertyReference { name, reference };
        self.properties.push(Box::new(property_reference));
        self
    }
    pub fn iter(&self) -> Iter<Box<Property<'a> + 'a>> {
        self.properties.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<Box<Property<'a> + 'a>> {
        self.properties.iter_mut()
    }
}

struct PropertyReference<'a, T>
where
    T: 'a,
{
    name: &'static str,
    reference: &'a mut T,
}

impl<'a, T> Property<'a> for PropertyReference<'a, T>
where
    T: Clone + From<Value> + Into<Value>,
{
    fn name(&self) -> &'static str {
        self.name
    }
    fn set(&mut self, value: Value) {
        *(self.reference) = value.into();
    }
    fn get(&self) -> Value {
        (*(self.reference)).clone().into()
    }
}

impl From<Value> for f32 {
    fn from(v: Value) -> f32 {
        match v {
            Value::Float(f) => f,
            v => panic!("Invalid cast from {:?} to f32", v),
        }
    }
}

impl Into<Value> for f32 {
    fn into(self) -> Value {
        Value::Float(self)
    }
}

impl From<Value> for bool {
    fn from(v: Value) -> bool {
        match v {
            Value::Bool(b) => b,
            v => panic!("Invalid cast from {:?} to bool", v),
        }
    }
}

impl Into<Value> for bool {
    fn into(self) -> Value {
        Value::Bool(self)
    }
}

impl From<Value> for i32 {
    fn from(v: Value) -> i32 {
        match v {
            Value::Integer(i) => i,
            v => panic!("Invalid cast from {:?} to i32", v),
        }
    }
}

impl Into<Value> for i32 {
    fn into(self) -> Value {
        Value::Integer(self)
    }
}
