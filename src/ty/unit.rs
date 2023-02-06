use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{BitSize, Endian};

/// 类型的大小与字节顺序
///
/// 默认`endian`为[`Endian::Big`]，`size`为[`None`]
/// **示例：**
/// ```rust
/// use bin2json::ty::{Endian, BitSize, Unit};
///
/// let unit: Unit = serde_json::from_str(r#"
/// {
///     "endian": "Little"
/// }
/// "#)?;
/// assert_eq!( Unit { endian: Endian::Little, size: None }, unit);
///
/// let unit: Unit = serde_json::from_str(r#"
/// {
///     "endian": "Little",
///     "size": { "type": "Bits", "value": 100 }
/// }
/// "#)?;
/// assert_eq!(Unit { endian: Endian::Little, size: Some(BitSize(100)) }, unit);
///
/// let unit: Unit = serde_json::from_str(r#"
/// {
///     "endian": "Big",
///     "size": { "type": "Bytes", "value": 200 }
/// }
/// "#)?;
/// assert_eq!(unit, Unit {endian: Endian::Big, size: Some(BitSize(200 * 8))});
/// # Ok::<_, serde_json::Error>(())
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    /// 字节顺序
    pub endian: Endian,
    /// 实际要读取的大小
    #[serde(default)]
    #[serde(serialize_with = "se_op_size")]
    #[serde(deserialize_with = "de_op_size")]
    pub size: Option<BitSize>,
}

impl Unit {
    pub fn new(endian: Endian, size: BitSize) -> Self {
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

impl From<Endian> for Unit {
    fn from(endian: Endian) -> Self {
        match endian {
            Endian::Big => Unit::big_endian(),
            Endian::Little => Unit::little_endian(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum SizeDef {
    Bits(usize),
    Bytes(usize),
}

fn se_op_size<S>(s: &Option<BitSize>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = match s {
        Some(BitSize(bits)) => Some(SizeDef::Bits(*bits)),
        None => None,
    };
    s.serialize(ser)
}

fn de_op_size<'de, D>(de: D) -> Result<Option<BitSize>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<SizeDef>::deserialize(de).map(|os| match os {
        Some(SizeDef::Bits(size)) => Some(BitSize(size)),
        Some(SizeDef::Bytes(size)) => Some(BitSize(size * 8)),
        None => None,
    })
}
