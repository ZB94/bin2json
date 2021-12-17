use crate::{BinToJson, BitSlice, BytesSize, get_data_by_size, Msb0, Type};
use crate::error::BinToJsonError;
use crate::Value;

#[derive(Debug, Clone)]
pub enum Length {
    Fixed(usize),
    By(String),
    None,
}

impl Length {
    pub fn by_field<S: Into<String>>(field: S) -> Self {
        Self::By(field.into())
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    /// 元素类型
    pub ty: Box<Type>,
    /// 数组长度
    pub length: Length,
    pub size: Option<BytesSize>,
}

impl Array {
    pub fn new(ty: Type) -> Self {
        Self {
            ty: Box::new(ty),
            length: Length::None,
            size: None,
        }
    }

    pub fn new_with_length(ty: Type, length: usize) -> Self {
        Self {
            ty: Box::new(ty),
            length: Length::Fixed(length),
            size: None,
        }
    }

    pub fn new_with_length_by<S: Into<String>>(ty: Type, by: S) -> Self {
        Self {
            ty: Box::new(ty),
            length: Length::by_field(by),
            size: None,
        }
    }
}

impl BinToJson for Array {
    fn read<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), BinToJsonError> {
        let src = data;
        let mut data = if let Some(size) = &self.size {
            get_data_by_size(data, size, None)?
        } else {
            data
        };
        let data_len = data.len();

        let (mut ret, len) = match &self.length {
            Length::Fixed(size) => (Vec::with_capacity(*size), *size),
            Length::None => (vec![], 0),
            Length::By(by) => return Err(BinToJsonError::ByKeyNotFound(by.clone()))
        };

        loop {
            match self.ty.read(data) {
                Ok((s, d)) => {
                    data = d;
                    ret.push(s);
                    if len > 0 && ret.len() == len {
                        break;
                    }
                }
                Err(_) => {
                    if len == 0 {
                        break;
                    } else {
                        return Err(BinToJsonError::Incomplete);
                    }
                }
            }
        }

        Ok((Value::Array(ret), &src[data_len - data.len()..]))
    }
}
