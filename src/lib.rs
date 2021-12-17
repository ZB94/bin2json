#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;

use std::collections::HashMap;

use deku::{DekuError, DekuRead};
use deku::bitvec::{BitSlice, Msb0};
pub use deku::bitvec;
use deku::ctx::Limit;

pub use _struct::Struct;
pub use array::{Array, Length};
pub use ty::{BytesSize, Endian, Size, Type, Unit};
pub use value::Value;

use crate::error::BinToJsonError;

pub mod error;
pub mod ty;

mod value;
mod _struct;
mod array;

#[cfg(test)]
mod tests;

pub trait BinToJson {
    fn read<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), BinToJsonError>;

    fn read_to_json<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(serde_json::Value, &'a BitSlice<Msb0, u8>), BinToJsonError> {
        self.read(data)
            .map(|(v, d)| (v.into(), d))
    }
}

pub(crate) fn get_data_by_size<'a>(
    data: &'a BitSlice<Msb0, u8>,
    size: &BytesSize,
    by_map: Option<&HashMap<String, Value>>,
) -> Result<&'a BitSlice<Msb0, u8>, BinToJsonError> {
    let len = match size {
        BytesSize::All => return Ok(data),
        BytesSize::Fixed(size) => *size,
        BytesSize::EndWith(with) => {
            let with_end_error = |e: DekuError| -> BinToJsonError {
                if let DekuError::Incomplete(_) = &e {
                    BinToJsonError::EndNotFound(with.clone())
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
        BytesSize::By(ref by) | BytesSize::Enum { ref by, .. } => {
            if let Some(map) = by_map {
                let by_value: serde_json::Value = map.get(by)
                    .cloned()
                    .ok_or(BinToJsonError::ByKeyNotFound(by.clone()))?
                    .into();

                if let BytesSize::Enum { map, .. } = size {
                    by_value.as_i64()
                        .and_then(|k| map.get(&k).copied())
                } else {
                    by_value.as_u64()
                        .map(|s| s as usize)
                }
                    .ok_or(BinToJsonError::LengthTargetIsInvalid(by.clone()))?
            } else {
                return Err(BinToJsonError::ByKeyNotFound(by.clone()));
            }
        }
    };

    let bits_len = len * 8;
    if data.len() >= bits_len {
        Ok(&data[..bits_len])
    } else {
        Err(BinToJsonError::Incomplete)
    }
}
