use std::collections::HashMap;

use deku::bitvec::BitSlice;

use crate::{Array, get_data_by_size, Length, Msb0, ReadBin, Type, Value};
use crate::error::ReadBinError;
use crate::ty::BytesSize;

/// 结构定义
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Struct {
    /// 结构的字段列表
    pub fields: Vec<Field>,
    /// 手动指定结构的总字节大小
    #[serde(default)]
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

/// 结构字段
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Field {
    /// 字段名称
    pub name: String,
    /// 字段类型
    #[serde(flatten)]
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

impl ReadBin for Struct {
    fn read<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), ReadBinError> {
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

            if let Type::Array(Array { length: Some(length), .. }) = &mut ty {
                if let Length::By(by) = length {
                    let by_value: serde_json::Value = ret.get(by)
                        .cloned()
                        .ok_or(ReadBinError::ByKeyNotFound(by.clone()))?
                        .into();

                    *length = Length::Fixed(
                        by_value.as_u64()
                            .ok_or(ReadBinError::LengthTargetIsInvalid(by.clone()))? as usize
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
                    .ok_or(ReadBinError::ByKeyNotFound(by.clone()))?
                    .as_i64()
                    .ok_or(ReadBinError::LengthTargetIsInvalid(by.clone()))?;

                ty = map.get(&key)
                    .cloned()
                    .ok_or(ReadBinError::EnumKeyNotFound(key))?;
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

