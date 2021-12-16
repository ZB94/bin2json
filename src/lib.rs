#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;

pub use _struct::Struct;
pub use array::Array;
pub use ty::{BytesSize, Endian, Size, Type, Unit};
pub use value::Value;

use crate::error::ParseError;

pub mod error;
pub mod ty;

mod value;
mod _struct;
mod array;

#[cfg(test)]
mod tests;

pub trait BinToJson {
    fn read<'a>(&self, data: &'a [u8]) -> Result<(Value, &'a [u8]), ParseError>;

    fn read_to_json<'a>(&self, data: &'a [u8]) -> Result<(serde_json::Value, &'a [u8]), ParseError> {
        self.read(data)
            .map(|(v, d)| (v.into(), d))
    }
}

pub(crate) fn get_data_by_size<'a>(data: &'a [u8], size: &Option<BytesSize>) -> Result<&'a [u8], ParseError> {
    match size {
        Some(BytesSize::By(_) | BytesSize::Enum { .. }) => Err(ParseError::ByKeyNotFound),
        Some(BytesSize::All) | None => Ok(data),
        Some(BytesSize::Fixed(size)) => if data.len() >= *size {
            Ok(&data[..*size])
        } else {
            Err(ParseError::Incomplete)
        }
        Some(BytesSize::EndWith(with)) => {
            let size = data.windows(with.len())
                .position(|w| w == with)
                .ok_or(ParseError::EndNotFound)?;
            if data.len() >= size {
                Ok(&data[..size])
            } else {
                Err(ParseError::Incomplete)
            }
        }
    }
}
