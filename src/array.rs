use std::collections::HashMap;

use crate::{BinToJson, Struct};
use crate::error::ParseError;
use crate::Value;

pub struct Array {
    /// 元素定义
    pub element: Struct,
    /// 数组长度，如果为`None`，则尽可能转换
    pub size: Option<usize>,
}

impl BinToJson for Array {
    type Output = Vec<HashMap<String, Value>>;

    fn read<'a>(&self, mut data: &'a [u8]) -> Result<(Self::Output, &'a [u8]), ParseError> {
        let mut ret = self.size.map(|s| Vec::with_capacity(s))
            .unwrap_or_default();

        let size = self.size.unwrap_or_default();
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

        Ok((ret, data))
    }

    fn read_to_json<'a>(&self, data: &'a [u8]) -> Result<(serde_json::Value, &'a [u8]), ParseError> {
        self.read(data)
            .map(|(l, d)| {
                let l = serde_json::Value::Array(l
                    .into_iter()
                    .map(|m| {
                        serde_json::Value::Object(
                            m.into_iter()
                                .map(|(k, v)| (k, v.into()))
                                .collect())
                    })
                    .collect());
                (l, d)
            })
    }
}
