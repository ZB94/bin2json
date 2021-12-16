use crate::{BinToJson, BytesSize, get_data_by_size, Type};
use crate::error::BinToJsonError;
use crate::Value;

#[derive(Debug, Clone)]
pub struct Array {
    /// 元素定义
    pub ty: Box<Type>,
    /// 数组长度，如果为`None`，则尽可能转换
    pub length: Option<usize>,
    pub size: Option<BytesSize>,
}

impl Array {
    pub fn new(ty: Type) -> Self {
        Self {
            ty: Box::new(ty),
            length: None,
            size: None,
        }
    }

    pub fn new_with_length(ty: Type, length: usize) -> Self {
        Self {
            ty: Box::new(ty),
            length: Some(length),
            size: None,
        }
    }
}

impl BinToJson for Array {
    fn read<'a>(&self, data: &'a [u8]) -> Result<(Value, &'a [u8]), BinToJsonError> {
        let mut data = get_data_by_size(data, &self.size)?;
        let mut ret = self.length.map(|s| Vec::with_capacity(s))
            .unwrap_or_default();

        let size = self.length.unwrap_or_default();
        loop {
            match self.ty.read(data) {
                Ok((s, d)) => {
                    data = d;
                    ret.push(s);
                    if size > 0 && ret.len() == size {
                        break;
                    }
                }
                Err(_) => {
                    if size == 0 {
                        break;
                    } else {
                        return Err(BinToJsonError::Incomplete);
                    }
                }
            }
        }

        Ok((Value::Array(ret), data))
    }
}
