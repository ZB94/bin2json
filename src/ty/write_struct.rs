use std::collections::HashMap;

use deku::bitvec::{BitVec, BitView, Msb0};
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
    let mut ret_cap = 0;

    for Field { name, ty } in fields {
        let value = match (ty, object.get(name)) {
            (Type::Checksum { .. }, _) => {
                result.push((name, ty, None));
                continue;
            }
            (Type::Magic { magic }, _) => {
                result.push((name, ty, Some(magic.view_bits().to_bitvec())));
                continue;
            }
            (_, None | Some(Value::Null)) => {
                result.push((name, ty, None));
                continue;
            }
            (_, Some(value)) => value
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
            = &mut ty
            {
                *size = None;
            }

            if let Type::Array { length: length @ Some(Length::By(_)), .. } = &mut ty {
                *length = None;
            }

            ty.write(value)?
        };
        ret_cap += bits.len();

        if let
        | Type::String { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }) }
        | Type::Bin { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }) }
        | Type::Struct { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }), .. }
        | Type::Array { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }), .. }
        | Type::Enum { size: Some(BytesSize::By(by) | BytesSize::Enum { by, .. }), .. }
        = ty
        {
            set_by_value(&mut result, ty, &bits, by)?;
        }

        if let (
            Type::Array { length: Some(Length::By(by)), .. },
            Some(l)
        ) = (ty, value.as_array()) {
            let (ty, out) = find_by(&mut result, by)?;
            *out = Some(ty.write(&(l.len().into()))?);
        }

        result.push((name, ty, Some(bits)));
    }

    let mut ret = BitVec::with_capacity(ret_cap);
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
