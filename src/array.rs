use crate::{BitSlice, BytesSize, get_data_by_size, Msb0, ReadBin, Type};
use crate::error::ReadBinError;
use crate::Value;

/// 数组长度
///
/// **示例：**
/// ```rust
/// use bin2json::Length;
///
/// let length: Length = serde_json::from_str(r#"100"#).unwrap();
/// assert_eq!(length, Length::Fixed(100));
///
/// let length: Length = serde_json::from_str(r#""field_name""#).unwrap();
/// assert_eq!(length, Length::By("field_name".to_string()));
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Length {
    /// 固定长度
    Fixed(usize),
    /// 通过指定字段指定。*使用该枚举时数据的定义应包含在结构体中，且指定的字段顺序应在数组之前*
    By(String),
}

impl Length {
    pub fn by_field<S: Into<String>>(field: S) -> Self {
        Self::By(field.into())
    }
}

/// 数组
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Array {
    /// 元素类型
    #[serde(flatten)]
    pub ty: Box<Type>,
    /// 数组长度
    #[serde(default)]
    pub length: Option<Length>,
    /// 手动指定数组的总字节大小
    #[serde(default, rename = "array_size")]
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
            length: Some(Length::Fixed(length)),
            size: None,
        }
    }

    pub fn new_with_length_by<S: Into<String>>(ty: Type, by: S) -> Self {
        Self {
            ty: Box::new(ty),
            length: Some(Length::by_field(by)),
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
            Some(Length::Fixed(size)) => (Vec::with_capacity(*size), *size),
            Some(Length::By(by)) => return Err(ReadBinError::ByKeyNotFound(by.clone())),
            None => (vec![], 0),
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
