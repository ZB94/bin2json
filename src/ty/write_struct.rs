use std::collections::HashMap;

use deku::bitvec::{BitVec, Msb0};
use serde_json::{Map, Value};

use crate::error::WriteBinError;
use crate::range::KeyRange;
use crate::ty::{BytesSize, Field, Length};
use crate::ty::utils::check_size;
use crate::Type;

pub fn write_struct(
    fields: &[Field],
    object: &Map<String, Value>,
) -> Result<BitVec<Msb0, u8>, WriteBinError> {
    let mut result = fields.iter()
        .map(|Field { name, ty }| (name, (ty, None)))
        .collect::<HashMap<_, (&Type, Option<BitVec<Msb0, u8>>)>>();

    // 重复原因：应对引用字段与Checksum或Sign嵌套
    for _ in 0..2 {
        let mut key_idx = HashMap::with_capacity(fields.len());
        for Field { name, ty } in fields {
            key_idx.insert(name, key_idx.len());

            if result[name].1.is_some() {
                continue;
            }

            let bits = match (ty, object.get(name)) {
                (
                    Type::Checksum { start_key, end_key, .. }
                    | Type::Sign { start_key, end_key, .. },
                    _
                ) => {
                    let end_key = end_key.as_ref().unwrap_or(name);
                    let start_idx = *key_idx.get(start_key).ok_or(WriteBinError::ByError)?;
                    let end_idx = *key_idx.get(end_key).ok_or(WriteBinError::ByError)?;

                    let mut bits_size = 0;
                    let l = fields[start_idx..end_idx].iter()
                        .map(|Field { name, .. }| {
                            if let Some(v) = &result[name].1 {
                                bits_size += v.len();
                                Ok(v)
                            } else {
                                Err(())
                            }
                        })
                        .collect::<Result<Vec<_>, _>>();
                    if l.is_err() {
                        continue;
                    }

                    let mut bits = BitVec::with_capacity(bits_size);
                    for v in l.unwrap() {
                        bits.extend_from_bitslice(v);
                    }

                    match ty {
                        Type::Checksum { method, .. } => {
                            if bits.len() % 8 != 0 {
                                return Err(WriteBinError::ChecksumError);
                            }
                            let bits = BitVec::from_vec(method.checksum(bits.as_raw_slice()));
                            Some(bits)
                        }
                        Type::Sign { on_write, size, .. } => {
                            let bits = on_write.sign(&bits)?;
                            check_size(size, &bits)?;
                            if let Some(BytesSize::By(by) | BytesSize::Enum { by, .. }) = size {
                                set_by_value(&mut result, ty, &bits, by)?;
                            }
                            Some(bits)
                        }
                        _ => None
                    }
                }
                (Type::Encrypt { inner_type, on_write, size, .. }, value) => {
                    let v = write_normal_field(inner_type, value, object, &mut result)?;
                    if let Some(data) = v {
                        let data = on_write.encrypt(data)?;
                        check_size(size, &data)?;

                        if let Some(BytesSize::By(by) | BytesSize::Enum { by, .. }) = size {
                            set_by_value(&mut result, ty, &data, by)?;
                        }

                        Some(data)
                    } else {
                        None
                    }
                }
                (_, value) => {
                    write_normal_field(ty, value, object, &mut result)?
                }
            };
            result.entry(name)
                .or_insert_with(|| (ty, None))
                .1 = bits;
        }
    }

    let mut bits_size = 0;
    let l = fields.iter()
        .map(|Field { name, .. }| {
            if let Some((_, Some(v))) = result.remove(name) {
                bits_size += v.len();
                Ok(v)
            } else {
                Err(WriteBinError::MissField(name.clone()))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut bits = BitVec::with_capacity(bits_size);
    for mut v in l {
        bits.append(&mut v);
    }

    Ok(bits)
}

fn write_normal_field(
    ty: &Type,
    value: Option<&Value>,
    object: &Map<String, Value>,
    result: &mut HashMap<&String, (&Type, Option<BitVec<Msb0, u8>>)>,
) -> Result<Option<BitVec<Msb0, u8>>, WriteBinError> {
    if let Type::Magic { .. } = ty {
        return ty.write(value.unwrap_or(&Value::Null))
            .map(|o| Some(o));
    }

    let value = if let Some(value) = value {
        value
    } else {
        return Ok(None);
    };

    let bits = if let Type::Enum { by, map, size } = ty {
        let key = object.get(by)
            .ok_or(WriteBinError::MissField(by.clone()))?
            .as_i64()
            .ok_or(WriteBinError::EnumByTypeError)?;
        let ty = map.get(&key)
            .ok_or(WriteBinError::EnumError)?;
        let out = ty.write(value)?;
        check_size(size, &out)?;
        out
    } else {
        let mut ty = ty.clone();

        if let
        | Type::String { size: size @ Some(BytesSize::By(_) | BytesSize::Enum { .. }) }
        | Type::Bin { size: size @ Some(BytesSize::By(_) | BytesSize::Enum { .. }) }
        | Type::Struct { size: size @ Some(BytesSize::By(_) | BytesSize::Enum { .. }), .. }
        | Type::Array { size: size @ Some(BytesSize::By(_) | BytesSize::Enum { .. }), .. }
        | Type::Encrypt { size: size @ Some(BytesSize::By(_) | BytesSize::Enum { .. }), .. }
        = &mut ty
        {
            *size = None;
        }

        if let Type::Array { length: length @ Some(Length::By(_)), .. } = &mut ty {
            *length = None;
        }

        ty.write(value)?
    };

    if let
    | Type::String { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }) }
    | Type::Bin { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }) }
    | Type::Struct { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }), .. }
    | Type::Array { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }), .. }
    | Type::Enum { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }), .. }
    | Type::Encrypt { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }), .. }
    = ty
    {
        set_by_value(result, ty, &bits, by)?;
    }

    if let (
        Type::Array { length: Some(Length::By(by)), .. },
        Some(l)
    ) = (ty, value.as_array()) {
        let (ty, out) = result.get_mut(by).ok_or(WriteBinError::ByError)?;
        *out = Some(ty.write(&(l.len().into()))?);
    }

    Ok(Some(bits))
}


fn set_by_value(
    result: &mut HashMap<&String, (&Type, Option<BitVec<Msb0, u8>>)>,
    ty: &Type,
    bits: &BitVec<Msb0, u8>,
    by: &String,
) -> Result<(), WriteBinError> {
    let bytes = bits.len() / 8;
    if bits.len() % 8 != 0 {
        return Err(WriteBinError::BytesSizeError);
    }

    let by_value = if let
    | Type::String { size: Some(BytesSize::Enum { map, .. }) }
    | Type::Bin { size: Some(BytesSize::Enum { map, .. }) }
    | Type::Struct { size: Some(BytesSize::Enum { map, .. }), .. }
    | Type::Array { size: Some(BytesSize::Enum { map, .. }), .. }
    | Type::Encrypt { size: Some(BytesSize::Enum { map, .. }), .. }
    = ty
    {
        if let Some(KeyRange::Value(k)) = map.find_key(&bytes) {
            k
        } else {
            return Err(WriteBinError::EnumError);
        }
    } else {
        bytes as i64
    };


    let (ty, out) = result.get_mut(by).ok_or(WriteBinError::ByError)?;
    *out = Some(ty.write(&(by_value.into()))?);
    Ok(())
}
