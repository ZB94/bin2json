use std::collections::HashMap;

use deku::bitvec::Msb0;
pub use deku::ctx::{Endian, Size};
use deku::ctx::Limit;
use deku::prelude::*;

pub use bytes_size::BytesSize;
pub use unit::Unit;

use crate::{Array, BitSlice, get_data_by_size, ReadBin, Struct};
use crate::_struct::Field;
use crate::error::ReadBinError;
use crate::Value;

mod bytes_size;
mod unit;

/// 数据类型
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Type {
    /// 魔法值。即一段固定数据值
    Magic(Vec<u8>),
    /// 布尔型数据。
    Boolean {
        /// 是否是位数据。如果是则进度去1比特位的数据作为该值，否则读取1字节。
        bit: bool
    },
    Int8(#[serde(default)] Unit),
    Int16(#[serde(default)] Unit),
    Int32(#[serde(default)] Unit),
    Int64(#[serde(default)] Unit),
    Uint8(#[serde(default)] Unit),
    Uint16(#[serde(default)] Unit),
    Uint32(#[serde(default)] Unit),
    Uint64(#[serde(default)] Unit),
    #[serde(with = "EndianDef")]
    Float32(Endian),
    #[serde(with = "EndianDef")]
    Float64(Endian),
    /// UTF8字符串
    String(BytesSize),
    /// 二进制数据
    Bin(BytesSize),
    /// 结构体
    Struct(Struct),
    /// 数组
    Array(Array),
    /// 枚举。
    ///
    /// **注意: **
    /// - 枚举类型不能单独存在，必须位于[`Struct`]的字段列表中
    Enum {
        /// 字段名称。*字段必须存在于与该枚举同级的结构体中，且指定的字段顺序应在该枚举之前*
        by: String,
        /// 枚举值对应的类型
        map: HashMap<i64, Type>,
        /// 手动指定枚举的总字节大小
        #[serde(default)]
        size: Option<BytesSize>,
    },
}

impl Type {
    /// 大小为1比特位的布尔值
    pub const BOOL_BIT: Type = Type::Boolean { bit: true };
    /// 大小为1字节的布尔值
    pub const BOOL: Type = Type::Boolean { bit: false };

    pub fn magic(magic: &[u8]) -> Self {
        Self::Magic(magic.to_vec())
    }
    pub fn int8() -> Self {
        Self::Int8(Default::default())
    }
    pub fn uint8() -> Self {
        Self::Uint8(Default::default())
    }
    pub fn new_struct(fields: Vec<Field>) -> Self {
        Self::Struct(Struct::new(fields))
    }
    pub fn new_array(ty: Type) -> Self {
        Self::Array(Array::new(ty))
    }
    pub fn new_enum<S: Into<String>>(by: S, map: HashMap<i64, Type>) -> Self {
        Self::Enum {
            by: by.into(),
            map,
            size: None,
        }
    }
}

macro_rules! parse_numeric_field {
    ($input: expr, $name: expr, $ty: ty, $unit: expr, $default_size: expr) => {{
        let size = $unit.size.unwrap_or($default_size);
        let (input, value) = <$ty>::read($input, ($unit.endian, size))?;
        (value.into(), input)
    }};
}

impl ReadBin for Type {
    fn read<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), ReadBinError> {
        let (value, data): (Value, _) = match self {
            Self::Magic(ref magic) => {
                let (input, value): (_, Vec<u8>) = DekuRead::read(
                    data,
                    Limit::new_count(magic.len()),
                )?;

                if magic == &value {
                    (value.into(), input)
                } else {
                    return Err(ReadBinError::MagicError(magic.clone()));
                }
            }
            Self::Boolean { bit } => {
                let size = if *bit { Size::Bits(1) } else { Size::Bytes(1) };
                let (input, v) = bool::read(data, size)?;
                (v.into(), input)
            }
            Self::Int8(unit) => {
                parse_numeric_field!(data, field.name, i8, unit, Size::Bytes(1))
            }
            Self::Int16(unit) => {
                parse_numeric_field!(data, field.name, i16, unit, Size::Bytes(2))
            }
            Self::Int32(unit) => {
                parse_numeric_field!(data, field.name, i32, unit, Size::Bytes(4))
            }
            Self::Int64(unit) => {
                parse_numeric_field!(data, field.name, i64, unit, Size::Bytes(8))
            }
            Self::Uint8(unit) => {
                parse_numeric_field!(data, field.name, u8, unit, Size::Bytes(1))
            }
            Self::Uint16(unit) => {
                parse_numeric_field!(data, field.name, u16, unit, Size::Bytes(2))
            }
            Self::Uint32(unit) => {
                parse_numeric_field!(data, field.name, u32, unit, Size::Bytes(4))
            }
            Self::Uint64(unit) => {
                parse_numeric_field!(data, field.name, u64, unit, Size::Bytes(8))
            }
            Self::Float32(endian) => {
                let (input, v) = f32::read(data, *endian)?;
                (v.into(), input)
            }
            Self::Float64(endian) => {
                let (input, v) = f64::read(data, *endian)?;
                (v.into(), input)
            }
            Self::String(ref size) | Type::Bin(ref size) => {
                let d = get_data_by_size(data, size, None)?;
                let d_len = d.len();

                let (_, v) = Vec::<u8>::read(data, Limit::new_size(Size::Bits(d_len)))?;
                let v = if let Type::String(_) = self {
                    String::from_utf8(v)?.into()
                } else {
                    v.into()
                };
                (v, &data[d_len..])
            }
            Self::Struct(s) => {
                s.read(data)?
            }
            Self::Array(a) => {
                a.read(data)?
            }
            Self::Enum { by, .. } => return Err(ReadBinError::ByKeyNotFound(by.clone())),
        };
        Ok((value, data))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Endian")]
enum EndianDef {
    Little,
    Big,
}
