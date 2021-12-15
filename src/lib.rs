#[macro_use]
extern crate thiserror;

use std::ops::{Deref, DerefMut};
use ty::Type;

pub mod error;
pub mod ty;

#[derive(Debug, Clone)]
pub struct Struct(pub Vec<Field>);

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

impl Struct {

}

impl Deref for Struct {
    type Target = Vec<Field>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Struct {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
