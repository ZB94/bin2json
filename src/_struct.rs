use std::collections::HashMap;

use deku::bitvec::BitSlice;

use crate::{Array, BinToJson, get_data_by_size, Length, Msb0, Type, Value};
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
    fn read<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), BinToJsonError> {
        let src = data;
        let mut data = if let Some(size) = &self.size {
            get_data_by_size(&data, size, None)?
        } else {
            data
        };
        let data_len = data.len();
        let mut ret: HashMap<String, Value> = HashMap::with_capacity(self.fields.len());

        for Field { name, ty } in &self.fields {
            let mut ty = ty.clone();

            if let Type::Array(Array { length, .. }) = &mut ty {
                if let Length::By(by) = length {
                    let by_value: serde_json::Value = ret.get(by)
                        .cloned()
                        .ok_or(BinToJsonError::ByKeyNotFound(by.clone()))?
                        .into();

                    *length = Length::Fixed(
                        by_value.as_u64()
                            .ok_or(BinToJsonError::LengthTargetIsInvalid(by.clone()))? as usize
                    )
                }
            }

            let (d, fixed_size) = if let Type::Bin(size)
            | Type::String(size)
            | Type::Array(Array { size: Some(size @ BytesSize::By(_) | size @ BytesSize::Enum { .. }), .. })
            | Type::Struct(Struct { size: Some(size @ BytesSize::By(_) | size @ BytesSize::Enum { .. }), .. })
            | Type::Enum { size: Some(size), .. }
            = &mut ty {
                let d = get_data_by_size(data, size, Some(&ret))?;
                *size = BytesSize::Fixed(d.len() / 8);
                (d, true)
            } else {
                (data, false)
            };

            if let Type::Enum { by, map, .. } = &ty {
                let key = ret.get(by)
                    .cloned()
                    .map::<serde_json::Value, _>(|v| v.into())
                    .ok_or(BinToJsonError::ByKeyNotFound(by.clone()))?
                    .as_i64()
                    .ok_or(BinToJsonError::LengthTargetIsInvalid(by.clone()))?;

                ty = map.get(&key)
                    .cloned()
                    .ok_or(BinToJsonError::EnumKeyNotFound(key))?;
            }

            let (v, d2) = ty.read(d)?;
            ret.insert(name.clone(), v);
            if fixed_size {
                data = &data[d.len()..];
            } else {
                data = &data[d.len() - d2.len()..];
            }
        }

        Ok((Value::Object(ret), &src[data_len - data.len()..]))
    }
}

