use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Endian, EndianDef, Size};

/// 类型的大小与字节顺序
///
/// **示例：**
/// ```rust
/// use bin2json::{Endian, Size, Unit};
///
/// let unit: Unit = serde_json::from_str(r#"
/// {
///     "endian": "Little"
/// }
/// "#).unwrap();
/// assert_eq!(unit, Unit {endian: Endian::Little, size: None});
///
/// let unit: Unit = serde_json::from_str(r#"
/// {
///     "endian": "Little",
///     "size": { "type": "Bits", "value": 100 }
/// }
/// "#).unwrap();
/// assert_eq!(unit, Unit {endian: Endian::Little, size: Some(Size::Bits(100))});
///
/// let unit: Unit = serde_json::from_str(r#"
/// {
///     "endian": "Big",
///     "size": { "type": "Bytes", "value": 200 }
/// }
/// "#).unwrap();
/// assert_eq!(unit, Unit {endian: Endian::Big, size: Some(Size::Bytes(200))});
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    /// 字节顺序
    #[serde(with = "EndianDef")]
    pub endian: Endian,
    /// 实际要读取的大小
    #[serde(default)]
    #[serde(serialize_with = "se_op_size")]
    #[serde(deserialize_with = "de_op_size")]
    pub size: Option<Size>,
}

impl Unit {
    pub fn new(endian: Endian, size: Size) -> Self {
        Self {
            endian,
            size: Some(size),
        }
    }

    pub const fn big_endian() -> Self {
        Self {
            endian: Endian::Big,
            size: None,
        }
    }

    pub const fn little_endian() -> Self {
        Self {
            endian: Endian::Little,
            size: None,
        }
    }
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            endian: Endian::Big,
            size: None,
        }
    }
}


#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum SizeDef {
    Bits(usize),
    Bytes(usize),
}

fn se_op_size<S>(s: &Option<Size>, ser: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let s = match s {
        Some(Size::Bits(size)) => Some(SizeDef::Bits(*size)),
        Some(Size::Bytes(size)) => Some(SizeDef::Bytes(*size)),
        None => None,
    };
    s.serialize(ser)
}

fn de_op_size<'de, D>(de: D) -> Result<Option<Size>, D::Error> where D: Deserializer<'de> {
    Option::<SizeDef>::deserialize(de)
        .map(|os| {
            match os {
                Some(SizeDef::Bits(size)) => Some(Size::Bits(size)),
                Some(SizeDef::Bytes(size)) => Some(Size::Bytes(size)),
                None => None,
            }
        })
}