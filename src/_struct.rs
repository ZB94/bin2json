use std::collections::HashMap;

use crate::{BinToJson, get_data_by_size, Type, Value};
use crate::error::BinToJsonError;
use crate::ty::BytesSize;

#[derive(Debug, Clone)]
pub struct Struct {
    pub fields: Vec<Field>,
    pub size: Option<BytesSize>,
}

impl Struct {
    pub fn new(fields: Vec<Field>) -> Self {
        Self {
            fields,
            size: None,
        }
    }

    pub fn new_with_size(fields: Vec<Field>, size: BytesSize) -> Self {
        Self {
            fields,
            size: Some(size),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

impl Field {
    pub fn new<S: Into<String>>(name: S, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

impl BinToJson for Struct {
    fn read<'a>(&self, data: &'a [u8]) -> Result<(Value, &'a [u8]), BinToJsonError> {
        let mut data = get_data_by_size(&data, &self.size)?;

        let mut ret: HashMap<String, Value> = HashMap::with_capacity(self.fields.len());

        for Field { name, ty } in &self.fields {
            let mut ty = ty.clone();
            match &mut ty {
                Type::Enum { by: key_by, map, size } => {
                    let d = get_data_by_size(data, size)?;
                    let d_len = d.len();

                    let key = ret.get(key_by)
                        .cloned()
                        .map::<serde_json::Value, _>(|v| v.into())
                        .ok_or(BinToJsonError::ByKeyNotFound)?
                        .as_i64()
                        .ok_or(BinToJsonError::LengthTargetIsInvalid)?;

                    let ty = map.get(&key)
                        .ok_or(BinToJsonError::EnumKeyNotFound)?;

                    let (v, d2) = ty.read(d)?;
                    ret.insert(name.clone(), v);
                    data = &data[d_len - d2.len()..];

                    continue;
                }
                Type::Bin(len) | Type::String(len) => {
                    if let Some(by) = len.by() {
                        *len = get_length_by_key(&ret, by, len)?;
                    }
                }
                _ => {}
            }
            let (v, d) = ty.read(data)?;
            ret.insert(name.clone(), v);
            data = d;
        }

        Ok((Value::Object(ret), data))
    }
}


fn get_length_by_key(map: &HashMap<String, Value>, by: &String, len: &BytesSize) -> Result<BytesSize, BinToJsonError> {
    let by_value: serde_json::Value = map.get(by)
        .cloned()
        .map(|v| v.into())
        .ok_or(BinToJsonError::ByKeyNotFound)?;

    let size = if let BytesSize::Enum { map, .. } = len {
        by_value.as_i64()
            .and_then(|k| map.get(&k).copied())
    } else {
        by_value.as_u64()
            .map(|s| s as usize)
    }
        .ok_or(BinToJsonError::LengthTargetIsInvalid)?;

    Ok(BytesSize::Fixed(size))
}
