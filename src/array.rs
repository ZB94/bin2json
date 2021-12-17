use crate::{ReadBin, BitSlice, BytesSize, get_data_by_size, Msb0, Type};
use crate::error::ReadBinError;
use crate::Value;

/// 数组长度
#[derive(Debug, Clone)]
pub enum Length {
    /// 固定长度
    Fixed(usize),
    /// 通过指定字段指定。*使用该枚举时数据的定义应包含在结构体中，且指定的字段顺序应在数组之前*
    By(String),
    /// 不限制
    ///
    /// 当数组使用该枚举值作为长度读取数据时，数组会不断尝试读取成员值，在读取出错或数据不足以继续读取时结束
    None,
}

impl Length {
    pub fn by_field<S: Into<String>>(field: S) -> Self {
        Self::By(field.into())
    }
}

/// 数组
#[derive(Debug, Clone)]
pub struct Array {
    /// 元素类型
    pub ty: Box<Type>,
    /// 数组长度
    pub length: Length,
    /// 手动指定数组的总字节大小
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

impl ReadBin for Array {
    fn read<'a>(&self, data: &'a BitSlice<Msb0, u8>) -> Result<(Value, &'a BitSlice<Msb0, u8>), ReadBinError> {
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
            Length::By(by) => return Err(ReadBinError::ByKeyNotFound(by.clone()))
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
                        return Err(ReadBinError::Incomplete);
                    }
                }
            }
        }

        Ok((Value::Array(ret), &src[data_len - data.len()..]))
    }
}
