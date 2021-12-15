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
            Bin(b) => b.into()
        }
    }
}
