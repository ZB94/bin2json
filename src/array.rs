use crate::{BinToJson, BytesSize, get_data_by_size, Struct};
use crate::error::ParseError;
use crate::Value;

pub struct Array {
    /// 元素定义
    pub element: Struct,
    /// 数组长度，如果为`None`，则尽可能转换
    pub length: Option<usize>,
    pub size: Option<BytesSize>,
}

impl BinToJson for Array {
    fn read<'a>(&self, data: &'a [u8]) -> Result<(Value, &'a [u8]), ParseError> {
        let mut data = get_data_by_size(data, &self.size)?;
        let mut ret = self.length.map(|s| Vec::with_capacity(s))
            .unwrap_or_default();

        let size = self.length.unwrap_or_default();
        loop {
            match self.element.read(data) {
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
                        return Err(ParseError::Incomplete);
                    }
                }
            }
        }

        Ok((Value::Array(ret), data))
    }
}
