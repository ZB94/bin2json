use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use deku::bitvec::BitView;
use deku::ctx::Limit;
use deku::DekuRead;

use crate::{BinToJson, Type, Value};
use crate::error::ParseError;
use crate::ty::{Length, Size};

#[derive(Debug, Clone)]
pub struct Struct(pub Vec<Field>);

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

macro_rules! parse_numeric_field {
    ($map: ident, $input: expr, $name: expr, $ty: ty, $unit: expr, $default_size: expr) => {{
        let size = $unit.size.unwrap_or($default_size);
        let (input, value) = <$ty>::read($input, ($unit.endian, size))?;
        $map.insert($name.clone(), value.into());
        input
    }};
}

impl BinToJson for Struct {
    type Output = HashMap<String, Value>;

    fn read<'a>(&self, data: &'a [u8]) -> Result<(HashMap<String, Value>, &'a [u8]), ParseError> {
        let mut ret: HashMap<String, Value> = HashMap::with_capacity(self.len());
        let mut data = data.view_bits();

        for field in &self.0 {
            data = match field.ty {
                Type::Magic(ref magic) => {
                    let (input, value): (_, Vec<u8>) = DekuRead::read(
                        data,
                        Limit::new_count(magic.len()),
                    )?;

                    if magic == &value {
                        ret.insert(field.name.clone(), value.into());
                        input
                    } else {
                        return Err(ParseError::MagicError);
                    }
                }
                Type::Boolean(unit) => {
                    parse_numeric_field!(ret, data, field.name, bool, unit, Size::Bytes(1))
                }
                Type::Int8(unit) => {
                    parse_numeric_field!(ret, data, field.name, i8, unit, Size::Bytes(1))
                }
                Type::Int16(unit) => {
                    parse_numeric_field!(ret, data, field.name, i16, unit, Size::Bytes(2))
                }
                Type::Int32(unit) => {
                    parse_numeric_field!(ret, data, field.name, i32, unit, Size::Bytes(4))
                }
                Type::Int64(unit) => {
                    parse_numeric_field!(ret, data, field.name, i64, unit, Size::Bytes(8))
                }
                Type::Uint8(unit) => {
                    parse_numeric_field!(ret, data, field.name, u8, unit, Size::Bytes(1))
                }
                Type::Uint16(unit) => {
                    parse_numeric_field!(ret, data, field.name, u16, unit, Size::Bytes(2))
                }
                Type::Uint32(unit) => {
                    parse_numeric_field!(ret, data, field.name, u32, unit, Size::Bytes(4))
                }
                Type::Uint64(unit) => {
                    parse_numeric_field!(ret, data, field.name, u64, unit, Size::Bytes(8))
                }
                Type::Float32(endian) => {
                    let (input, v) = f32::read(data, endian)?;
                    ret.insert(field.name.clone(), v.into());
                    input
                }
                Type::Float64(endian) => {
                    let (input, v) = f64::read(data, endian)?;
                    ret.insert(field.name.clone(), v.into());
                    input
                }
                Type::String(ref len) | Type::Bin(ref len) => {
                    let (input, v): (_, Vec<u8>) = match len {
                        Length::All => {
                            DekuRead::read(data, Limit::new_size(Size::Bits(data.len())))?
                        }
                        Length::Fixed(len) => {
                            DekuRead::read(data, Limit::new_count(*len))?
                        }
                        Length::EndWith(with) => {
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
                        Length::By(by) | Length::Enum { by, .. } => {
                            let by_value: serde_json::Value = ret.get(by)
                                .cloned()
                                .map(|v| v.into())
                                .ok_or(ParseError::ByKeyNotFound)?;
                            let size = if let Length::Enum { map, .. } = len {
                                by_value.as_i64()
                                    .and_then(|k| map.get(&(k as isize)).copied())
                            } else {
                                by_value.as_u64()
                                    .map(|s| s as usize)
                            }
                                .ok_or(ParseError::LengthTargetIsInvalid)?;
                            DekuRead::read(data, Limit::new_count(size))?
                        }
                    };

                    if let Type::String(_) = field.ty {
                        ret.insert(field.name.clone(), String::from_utf8(v)?.into());
                    } else {
                        ret.insert(field.name.clone(), v.into());
                    }

                    input
                }
            }
        }

        Ok((ret, data.as_raw_slice()))
    }

    fn read_to_json<'a>(&self, data: &'a [u8]) -> Result<(serde_json::Value, &'a [u8]), ParseError> {
        let mut map = serde_json::Map::with_capacity(self.len());
        let (m, data) = self.read(data)?;
        for (k, v) in m {
            map.insert(k, v.into());
        }
        Ok((serde_json::Value::Object(map), data))
    }
}

impl Deref for Struct {
    type Target = Vec<Field>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Struct {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
