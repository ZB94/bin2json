use std::collections::HashMap;
use std::ops::Index;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Float32(f32),
    Float64(f64),
    String(String),
    Bin(Vec<u8>),
    Array(Vec<Self>),
    Object(HashMap<String, Self>),
}

impl Into<serde_json::Value> for Value {
    fn into(self) -> serde_json::Value {
        use crate::value::Value::*;
        match self {
            Boolean(b) => b.into(),
            Int8(n) => n.into(),
            Int16(n) => n.into(),
            Int32(n) => n.into(),
            Int64(n) => n.into(),
            Uint8(n) => n.into(),
            Uint16(n) => n.into(),
            Uint32(n) => n.into(),
            Uint64(n) => n.into(),
            Float32(n) => n.into(),
            Float64(n) => n.into(),
            String(s) => s.into(),
            Bin(b) => b.into(),
            Array(a) => serde_json::Value::Array(a.into_iter()
                .map(|v| v.into())
                .collect()),
            Object(m) => serde_json::Value::Object(m.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect())
        }
    }
}

macro_rules! impl_from {
    ($from: ty, $target: path) => {
        impl From<$from> for Value {
            fn from(value: $from) -> Self {
                $target(value)
            }
        }
    };
}

impl_from!(bool, Value::Boolean);
impl_from!(i8, Value::Int8);
impl_from!(i16, Value::Int16);
impl_from!(i32, Value::Int32);
impl_from!(i64, Value::Int64);
impl_from!(u8, Value::Uint8);
impl_from!(u16, Value::Uint16);
impl_from!(u32, Value::Uint32);
impl_from!(u64, Value::Uint64);
impl_from!(f32, Value::Float32);
impl_from!(f64, Value::Float64);
impl_from!(String, Value::String);
impl_from!(Vec<u8>, Value::Bin);

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Self::Bin(value.to_vec())
    }
}

impl<const L: usize> From<&[u8; L]> for Value {
    fn from(value: &[u8; L]) -> Self {
        Self::Bin(value.to_vec())
    }
}

impl From<Vec<Value>> for Value {
    fn from(l: Vec<Value>) -> Self {
        Self::Array(l)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(m: HashMap<String, Value>) -> Self {
        Self::Object(m)
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        if let Value::Array(a) = self {
            a.index(index)
        } else {
            panic!("当前Value的类型不是数组")
        }
    }
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        if let Value::Object(o) = self {
            o.index(index)
        } else {
            panic!("当前Value的类型不是对象")
        }
    }
}
