use std::collections::HashMap;

use deku::bitvec::{BitVec, Msb0};
use deku::ctx::{Limit, Size};
use deku::DekuRead;
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
    let mut result = Vec::with_capacity(fields.len());

    for Field { name, ty } in fields {
        match (ty, object.get(name)) {
            (Type::Checksum { .. }, _) => result.push((name, ty, None)),
            (Type::Encrypt { inner_type, on_write, size, .. }, value) => {
                let v = write_normal_field(inner_type, value, object, &mut result)?;
                let v = if let Some(data) = v {
                    let data = on_write.encrypt(data)?;
                    check_size(size, &data)?;

                    if let Some(BytesSize::By(by) | BytesSize::Enum { by, .. }) = size {
                        set_by_value(&mut result, ty, &data, by)?;
                    }

                    Some(data)
                } else {
                    None
                };
                result.push((name, ty, v));
            }
            (_, value) => {
                let v = write_normal_field(ty, value, object, &mut result)?;
                result.push((name, ty, v));
            }
        };
    }

    let mut ret = BitVec::new();
    let mut key_pos = HashMap::with_capacity(fields.len());
    for (k, ty, v) in result {
        key_pos.insert(k, ret.len());

        if let Type::Checksum { method, start_key, end_key } = ty {
            let end_key = end_key.as_ref().unwrap_or(k);
            let start_pos = key_pos[start_key];
            let end_pos = key_pos[end_key];
            let size = end_pos - start_pos;
            if size == 0 || size % 8 != 0 {
                return Err(WriteBinError::ChecksumError);
            }
            let (_, data) = Vec::<u8>::read(
                &ret[start_pos..end_pos],
                Limit::new_size(Size::Bits(size)),
            )?;
            let checksum = method.checksum(&data);
            ret.extend_from_raw_slice(&checksum);
        } else {
            ret.append(&mut v.ok_or(WriteBinError::MissField(k.clone()))?);
        };
    }
    Ok(ret)
}

fn write_normal_field(
    ty: &Type,
    value: Option<&Value>,
    object: &Map<String, Value>,
    result: &mut Vec<(&String, &Type, Option<BitVec<Msb0, u8>>)>,
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
        let (ty, out) = find_by(result, by)?;
        *out = Some(ty.write(&(l.len().into()))?);
    }

    Ok(Some(bits))
}


fn set_by_value(
    result: &mut Vec<(&String, &Type, Option<BitVec<Msb0, u8>>)>,
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


    let (ty, out) = find_by(result, by)?;
    *out = Some(ty.write(&(by_value.into()))?);
    Ok(())
}

fn find_by<'a>(result: &'a mut Vec<(&String, &Type, Option<BitVec<Msb0, u8>>)>, by: &String) -> Result<(&'a Type, &'a mut Option<BitVec<Msb0, u8>>), WriteBinError> {
    for (name, ty, out) in result {
        if name == &by {
            return Ok((ty, out));
        }
    }

    Err(WriteBinError::ByError)
}
