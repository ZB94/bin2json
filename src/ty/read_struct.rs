use std::collections::HashMap;

use deku::bitvec::{BitSlice, Msb0};
use deku::ctx::{Limit, Size};
use deku::DekuRead;
use serde_json::Map;

use crate::{ReadBinError, Type, Value};
use crate::ty::{BytesSize, Field, Length};
use crate::ty::utils::get_data_by_size;

pub fn read_struct<'a>(
    fields: &[Field],
    size: &Option<BytesSize>,
    data: &'a BitSlice<Msb0, u8>,
) -> Result<(Value, &'a BitSlice<Msb0, u8>), ReadBinError> {
    let src = data;
    let mut data = get_data_by_size(&data, size, None)?;
    let data_len = data.len();
    let mut ret: Map<String, Value> = Map::with_capacity(fields.len());
    let mut key_pos: HashMap<&String, usize> = HashMap::with_capacity(fields.len());

    for Field { name, ty } in fields {
        key_pos.insert(name, src.len() - data.len());

        data = match ty {
            Type::Checksum { method, start_key, end_key } => {
                // 读取校验和
                let (checksum, data) = method.read(data)?;

                // 获取要计算校验和的数据
                let end_key = end_key.as_ref().unwrap_or(name);
                let checksum_data = read_sub_data(
                    start_key,
                    end_key,
                    &key_pos,
                    src,
                    ReadBinError::ChecksumError,
                )?;

                // 检查校验和
                if method.check(&checksum_data, &checksum) {
                    ret.insert(name.to_string(), checksum.into());
                    data
                } else {
                    return Err(ReadBinError::ChecksumError);
                }
            }
            Type::Sign { start_key, end_key, on_read, size, .. } => {
                let signed_data = get_data_by_size(data, size, Some(&ret))?;
                if signed_data.len() % 8 != 0 {
                    return Err(ReadBinError::VerifyError(format!("已签名数据必须全部为完整字节")));
                }
                let (_, sd) = Vec::<u8>::read(
                    signed_data,
                    Limit::new_size(Size::Bits(signed_data.len())),
                )?;

                let end_key = end_key.as_ref().unwrap_or(name);
                let sign_data = read_sub_data(
                    start_key,
                    end_key,
                    &key_pos,
                    src,
                    ReadBinError::VerifyError(format!("待验证数据必须全部为完整字节")),
                )?;
                on_read.verify(&sign_data, &sd)?;
                ret.insert(name.clone(), sd.into());
                &data[signed_data.len()..]
            }
            Type::Encrypt { inner_type, on_read, size, .. } => {
                let en_data = get_data_by_size(data, size, Some(&ret))?;
                let de_data = on_read.decrypt(en_data)?;
                read_normal_field(name, inner_type, &de_data, &mut ret)?;
                &data[en_data.len()..]
            }
            _ => read_normal_field(name, ty, data, &mut ret)?,
        };
    }

    Ok((Value::Object(ret), &src[data_len - data.len()..]))
}

fn read_normal_field<'a>(
    name: &String,
    ty: &Type,
    data: &'a BitSlice<Msb0, u8>,
    result: &mut Map<String, Value>,
) -> Result<&'a BitSlice<Msb0, u8>, ReadBinError> {
    let mut ty = ty.clone();
    if let Type::Array { length: Some(length), .. } = &mut ty {
        if let Length::By(by) = length {
            let len = result.get(by)
                .ok_or(ReadBinError::ByKeyNotFound(by.clone()))?
                .as_u64()
                .ok_or(ReadBinError::LengthTargetIsInvalid(by.clone()))? as usize;
            *length = Length::Fixed(len)
        }
    }

    let (d, fixed_size) = if let
    | Type::Bin { size }
    | Type::String { size }
    | Type::Array { size, .. }
    | Type::Struct { size, .. }
    | Type::Enum { size, .. }
    | Type::Encrypt { size, .. }
    = &mut ty {
        let fs = size.is_some();
        let d = get_data_by_size(data, size, Some(&result))?;
        *size = Some(BytesSize::Fixed(d.len() / 8));
        (d, fs)
    } else {
        (data, false)
    };

    if let Type::Enum { by, map, .. } = &ty {
        let key = result.get(by)
            .ok_or(ReadBinError::ByKeyNotFound(by.clone()))?
            .as_i64()
            .ok_or(ReadBinError::LengthTargetIsInvalid(by.clone()))?;

        ty = map.get(&key)
            .cloned()
            .ok_or(ReadBinError::EnumKeyNotFound(key))?;
    }

    let (v, d2) = ty.read(d)?;
    result.insert(name.clone(), v);
    if fixed_size {
        Ok(&data[d.len()..])
    } else {
        Ok(&data[d.len() - d2.len()..])
    }
}

pub fn read_sub_data(
    start_key: &String,
    end_key: &String,
    key_pos: &HashMap<&String, usize>,
    src: &BitSlice<Msb0, u8>,
    on_size_error: ReadBinError,
) -> Result<Vec<u8>, ReadBinError> {
    let start_pos = *key_pos.get(start_key)
        .ok_or(ReadBinError::ByKeyNotFound(start_key.clone()))?;
    let end_pos = *key_pos.get(end_key)
        .ok_or(ReadBinError::ByKeyNotFound(end_key.clone()))?;

    let size = end_pos - start_pos;
    if size == 0 || size % 8 != 0 {
        return Err(on_size_error);
    }

    let (_, data) = Vec::<u8>::read(
        &src[start_pos..end_pos],
        Limit::new_size(Size::Bits(size)),
    )?;

    Ok(data)
}
