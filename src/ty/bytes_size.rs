use crate::range::KeyRangeMap;

/// 总字节大小
///
/// **示例：**
/// ```rust
/// use bin2json::ty::BytesSize;
///
/// let bs: BytesSize = serde_json::from_str(r#"100"#).unwrap();
/// assert_eq!(bs, BytesSize::Fixed(100));
///
/// let bs: BytesSize = serde_json::from_str(r#"[1, 2, 3]"#).unwrap();
/// assert_eq!(bs, BytesSize::EndWith(vec![1, 2, 3]));
///
/// let bs: BytesSize = serde_json::from_str(r#""field name""#).unwrap();
/// assert_eq!(bs, BytesSize::By("field name".to_string()));
///
/// let bs: BytesSize = serde_json::from_str(r#"
/// {
///     "by": "field name",
///     "map": {
///         "1": 2,
///         "3..": 4
///     }
/// }"#).unwrap();
/// assert_eq!(bs, BytesSize::Enum {
///     by: "field name".to_string(),
///     map: bin2json::range_map!(1 => 2, 3.. => 4)
/// });
///
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BytesSize {
    /// 固定长度
    Fixed(usize),
    /// 以指定数据结尾
    EndWith(Vec<u8>),
    /// 通过指定字段的值。指定字段的类型必须为整数
    By(String),
    /// 根据指定字段的值有不同的大小，指定字段的类型必须为整数
    Enum {
        /// 字段名称
        by: String,
        /// 键为指定字段的值，值为大小
        map: KeyRangeMap<usize>,
    },
}

impl BytesSize {
    pub fn by_enum<S: Into<String>, M: Into<KeyRangeMap<usize>>>(target_field: S, map: M) -> Self {
        Self::Enum {
            by: target_field.into(),
            map: map.into(),
        }
    }

    pub fn new<BS: Into<BytesSize>>(bs: BS) -> Self {
        bs.into()
    }
}

impl From<usize> for BytesSize {
    fn from(size: usize) -> Self {
        Self::Fixed(size)
    }
}

impl From<String> for BytesSize {
    fn from(by: String) -> Self {
        Self::By(by)
    }
}

impl From<&str> for BytesSize {
    fn from(by: &str) -> Self {
        by.to_string().into()
    }
}

impl From<Vec<u8>> for BytesSize {
    fn from(end: Vec<u8>) -> Self {
        Self::EndWith(end)
    }
}

impl From<&[u8]> for BytesSize {
    fn from(end: &[u8]) -> Self {
        end.to_vec().into()
    }
}

impl<const L: usize> From<&[u8; L]> for BytesSize {
    fn from(end: &[u8; L]) -> Self {
        end.to_vec().into()
    }
}
