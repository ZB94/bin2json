use deku::bitvec::{BitVec, Msb0};
use serde_json::{Map, Value};
use crate::error::WriteBinError;

use crate::Type;
use crate::range::KeyRange;
use crate::ty::{BytesSize, Field, Length};
use crate::ty::utils::check_size;

pub fn write_struct(
    fields: &[Field],
    object: &Map<String, Value>,
) -> Result<BitVec<Msb0, u8>, WriteBinError> {
    let mut result = Vec::with_capacity(fields.len());
    let mut ret_cap = 0;

    for Field { name, ty } in fields {
        let value = object.get(name);
        if value.map(Value::is_null).unwrap_or(true) {
            result.push((name, ty, None));
            continue;
        }

        let value = value.unwrap();

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
    for (k, _, v) in result {
        ret.append(&mut v.ok_or(WriteBinError::MissField(k.clone()))?);
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
