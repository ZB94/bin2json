#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;

use std::collections::HashMap;

use deku::{DekuError, DekuRead};
use deku::bitvec::{BitSlice, Msb0};
pub use deku::bitvec;
use deku::ctx::Limit;

pub use ty::{BytesSize, Endian, Field, Size, Type, Unit};
pub use value::Value;

use crate::error::ReadBinError;

pub mod error;
pub mod ty;

mod value;
pub mod range;

#[cfg(test)]
mod tests;

/// 从二进制读取数据
pub trait ReadBin {
    /// 从数据中读取数据并返回读取到的值和未使用的数据
    fn read<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), ReadBinError>;

    /// 与[`ReadBin::read`]类似，但是返回的值是[`serde_json::Value`]
    fn read_to_json<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(serde_json::Value, &'a BitSlice<Msb0, u8>), ReadBinError> {
        self.read(data)
            .map(|(v, d)| (v.into(), d))
    }
}

pub(crate) fn get_data_by_size<'a>(
    data: &'a BitSlice<Msb0, u8>,
    size: &Option<BytesSize>,
    by_map: Option<&HashMap<String, Value>>,
) -> Result<&'a BitSlice<Msb0, u8>, ReadBinError> {
    let len = match size {
        None => return Ok(data),
        Some(BytesSize::Fixed(size)) => *size,
        Some(BytesSize::EndWith(with)) => {
            let with_end_error = |e: DekuError| -> ReadBinError {
                if let DekuError::Incomplete(_) = &e {
                    ReadBinError::EndNotFound(with.clone())
                } else {
                    e.into()
                }
            };

            let (mut d, mut v) = Vec::<u8>::read(data, Limit::new_count(with.len()))
                .map_err(with_end_error)?;
            while !v.ends_with(with) {
                let (d2, b) = u8::read(d, ()).map_err(with_end_error)?;
                v.push(b);
                d = d2;
            }
            v.len()
        }
        Some(BytesSize::By(ref by) | BytesSize::Enum { ref by, .. }) => {
            if let Some(map) = by_map {
                let by_value: serde_json::Value = map.get(by)
                    .cloned()
                    .ok_or(ReadBinError::ByKeyNotFound(by.clone()))?
                    .into();

                if let Some(BytesSize::Enum { map, .. }) = size {
                    by_value.as_i64()
                        .and_then(|key| map.get(&key).copied())
                } else {
                    by_value.as_u64()
                        .map(|s| s as usize)
                }
                    .ok_or(ReadBinError::LengthTargetIsInvalid(by.clone()))?
            } else {
                return Err(ReadBinError::ByKeyNotFound(by.clone()));
            }
        }
    };

    let bits_len = len * 8;
    if data.len() >= bits_len {
        Ok(&data[..bits_len])
    } else {
        Err(ReadBinError::Incomplete)
    }
}
