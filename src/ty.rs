use std::collections::HashMap;

use deku::bitvec::BitView;
pub use deku::ctx::{Endian, Size};
use deku::ctx::Limit;
use deku::prelude::*;

use crate::{Array, BinToJson, Struct};
use crate::error::ParseError;
use crate::Value;

#[derive(Debug, Clone)]
pub enum Type {
    Magic(Vec<u8>),
    Boolean(Unit),
    Int8(Unit),
    Int16(Unit),
    Int32(Unit),
    Int64(Unit),
    Uint8(Unit),
    Uint16(Unit),
    Uint32(Unit),
    Uint64(Unit),
    Float32(Endian),
    Float64(Endian),
    String(BytesSize),
    Bin(BytesSize),
    Struct(Struct),
    Array(Array),
}

macro_rules! parse_numeric_field {
    ($input: expr, $name: expr, $ty: ty, $unit: expr, $default_size: expr) => {{
        let size = $unit.size.unwrap_or($default_size);
        let (input, value) = <$ty>::read($input, ($unit.endian, size))?;
        (value.into(), input)
    }};
}

impl BinToJson for Type {
    fn read<'a>(&self, data: &'a [u8]) -> Result<(Value, &'a [u8]), ParseError> {
        let data = data.view_bits();
        let (value, data): (Value, _) = match self {
            Self::Magic(ref magic) => {
                let (input, value): (_, Vec<u8>) = DekuRead::read(
                    data,
                    Limit::new_count(magic.len()),
                )?;

                if magic == &value {
                    (value.into(), input)
                } else {
                    return Err(ParseError::MagicError);
                }
            }
            Self::Boolean(unit) => {
                parse_numeric_field!(data, field.name, bool, unit, Size::Bytes(1))
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
            Self::String(ref len) | Type::Bin(ref len) => {
                let (input, v): (_, Vec<u8>) = match len {
                    BytesSize::All => {
                        DekuRead::read(data, Limit::new_size(Size::Bits(data.len())))?
                    }
                    BytesSize::Fixed(len) => {
                        DekuRead::read(data, Limit::new_count(*len))?
                    }
                    BytesSize::EndWith(with) => {
                        let (mut i, mut d): (_, Vec<u8>) = DekuRead::read(
                            data,
                            Limit::new_count(with.len()),
                        )?;
                        while !d.ends_with(with) {
                            let (i2, b) = u8::read(i, ())
                                .map_err(|_| ParseError::EndNotFound)?;
                            i = i2;
                            d.push(b);
                        }
                        (i, d)
                    }
                    BytesSize::By(_) | BytesSize::Enum { .. } => {
                        return Err(ParseError::ByKeyNotFound);
                    }
                };

                let v = if let Type::String(_) = self {
                    String::from_utf8(v)?.into()
                } else {
                    v.into()
                };
                (v, input)
            }
            Self::Struct(s) => {
                s.read(data.as_raw_slice())
                    .map(|(v, d)| (v, d.view_bits()))?
            }
            Self::Array(a) => {
                a.read(data.as_raw_slice())
                    .map(|(v, d)| (v, d.view_bits()))?
            }
        };
        Ok((value, data.as_raw_slice()))
    }
}

impl Type {
    pub fn magic(magic: &[u8]) -> Self {
        Self::Magic(magic.to_vec())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Unit {
    /// 字节顺序
    pub endian: Endian,
    /// 实际要读取的大小
    pub size: Option<Size>,
}

impl Unit {
    pub fn new(endian: Endian, size: Option<Size>) -> Self {
        Self {
            endian,
            size,
        }
    }

    pub const fn big_endian() -> Self {
        Self {
            endian: Endian::Big,
            size: None,
        }
    }

    pub const fn little_endian() -> Self {
        Self {
            endian: Endian::Little,
            size: None,
        }
    }
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            endian: Endian::Big,
            size: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BytesSize {
    /// 所有数据
    All,
    /// 固定长度
    Fixed(usize),
    /// 以指定数据结尾
    EndWith(Vec<u8>),
    /// 通过指定字段的值。指定字段的类型必须为整数
    By(String),
    /// 根据指定字段的值有不同的大小，指定字段的类型必须为整数
    Enum {
        /// 字段名称
        by: String,
        /// 键为指定字段的值，值为大小
        map: HashMap<isize, usize>,
    },
}

impl BytesSize {
    pub fn by_enum<S: Into<String>>(target_field: S, map: HashMap<isize, usize>) -> Self {
        Self::Enum {
            by: target_field.into(),
            map,
        }
    }

    pub fn by_field<S: Into<String>>(target: S) -> Self {
        Self::By(target.into())
    }

    pub fn by(&self) -> Option<&String> {
        if let Self::By(name) | Self::Enum { by: name, .. } = self {
            Some(name)
        } else {
            None
        }
    }
}
