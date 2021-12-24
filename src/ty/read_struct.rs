use deku::bitvec::{BitSlice, Msb0};
use serde_json::Map;

use crate::{ReadBinError, Type, Value};
use crate::ty::{BytesSize, Field, Length};
use crate::ty::utils::get_data_by_size;

pub fn read_struct<'a>(fields: &[Field], size: &Option<BytesSize>, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), ReadBinError> {
    let src = data;
    let mut data = get_data_by_size(&data, size, None)?;
    let data_len = data.len();
    let mut ret: Map<String, Value> = Map::with_capacity(fields.len());

    for Field { name, ty } in fields {
        let mut ty = ty.clone();
        if let Type::Array { length: Some(length), .. } = &mut ty {
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

        let (d, fixed_size) = if let Type::Bin { size }
        | Type::String { size }
        | Type::Array { size, .. }
        | Type::Struct { size, .. }
        | Type::Enum { size, .. }
        = &mut ty {
            let fs = size.is_some();
            let d = get_data_by_size(data, size, Some(&ret))?;
            *size = Some(BytesSize::Fixed(d.len() / 8));
            (d, fs)
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
