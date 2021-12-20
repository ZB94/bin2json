use std::collections::HashMap;
use std::fmt::Formatter;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;

/// 总字节大小
///
/// **示例：**
/// ```rust
/// use bin2json::BytesSize;
///
/// let bs: BytesSize = serde_json::from_str(r#""all""#).unwrap();
/// assert_eq!(bs, BytesSize::All);
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
///         "3": 4
///     }
/// }"#).unwrap();
/// assert_eq!(bs, BytesSize::Enum {
///     by: "field name".to_string(),
///     map: [(1, 2), (3, 4)].into_iter().collect()
/// });
///
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BytesSize {
    /// 所有数据
    All,
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
        map: HashMap<i64, usize>,
    },
}

impl BytesSize {
    pub fn by_enum<S: Into<String>>(target_field: S, map: HashMap<i64, usize>) -> Self {
        Self::Enum {
            by: target_field.into(),
            map,
        }
    }

    pub fn by_field<S: Into<String>>(target: S) -> Self {
        Self::By(target.into())
    }
}

impl Serialize for BytesSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            BytesSize::All => "all".serialize(serializer),
            BytesSize::Fixed(size) => size.serialize(serializer),
            BytesSize::EndWith(end) => end.serialize(serializer),
            BytesSize::By(by) => by.serialize(serializer),
            BytesSize::Enum { by, map } => {
                let mut sm = serializer.serialize_map(Some(2))?;
                sm.serialize_entry("by", by)?;
                sm.serialize_entry("map", map)?;
                sm.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for BytesSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_any(ByteSizeVisitor)
    }
}

struct ByteSizeVisitor;

impl<'de> Visitor<'de> for ByteSizeVisitor {
    type Value = BytesSize;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "ByteSize的值必须为非负整数、字符串、单字节数组或者包含`by`和`map`两个键的对象")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: Error {
        Ok(BytesSize::Fixed(v as usize))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        if v == "all" {
            Ok(BytesSize::All)
        } else {
            Ok(BytesSize::By(v.to_string()))
        }
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: Error {
        Ok(BytesSize::EndWith(v.to_vec()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut bytes = vec![];
        while let Some(b) = seq.next_element::<u8>()? {
            bytes.push(b);
        }
        Ok(BytesSize::EndWith(bytes))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
        let mut b = None;
        let mut m = None;

        while let Some(k) = map.next_key::<String>()? {
            match k.as_str() {
                "by" => {
                    if b.is_none() {
                        b = Some(map.next_value::<String>()?)
                    } else {
                        return Err(A::Error::duplicate_field("by"));
                    }
                }
                "map" => {
                    if m.is_none() {
                        m = Some(map.next_value::<HashMap<i64, usize>>()?)
                    } else {
                        return Err(A::Error::duplicate_field("map"));
                    }
                }
                _ => return Err(A::Error::unknown_field(&k, &["by", "enum"])),
            }
        };

        let by = b.ok_or(A::Error::missing_field("by"))?;
        let map = m.ok_or(A::Error::missing_field("map"))?;
        Ok(BytesSize::Enum { by, map })
    }
}
