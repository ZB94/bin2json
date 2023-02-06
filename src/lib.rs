#![doc = include_str ! ("../README.md")]

#[macro_use]
extern crate serde;
pub extern crate serde_json;
#[macro_use]
extern crate thiserror;

pub use deku::bitvec;
pub use serde_json::Value;

pub use error::ReadBinError;
pub use ty::Type;

pub mod error;
pub mod range;
pub mod secure;
pub mod ty;

#[cfg(test)]
mod tests;
