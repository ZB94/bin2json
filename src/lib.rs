#[macro_use]
extern crate thiserror;


pub use _struct::{Field, Struct};
pub use array::Array;
pub use ty::{Endian, Length, Size, Type, Unit};
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
    type Output;

    fn read<'a>(&self, data: &'a [u8]) -> Result<(Self::Output, &'a [u8]), ParseError>;
    fn read_to_json<'a>(&self, data: &'a [u8]) -> Result<(serde_json::Value, &'a [u8]), ParseError>;
}
