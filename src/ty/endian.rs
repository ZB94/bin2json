
/// 字节顺序
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Endian {
    /// 大段
    Big,
    /// 小端
    Little,
}

impl Into<deku::ctx::Endian> for Endian {
    fn into(self) -> deku::ctx::Endian {
        match self {
            Self::Little => deku::ctx::Endian::Little,
            Self::Big => deku::ctx::Endian::Big,
        }
    }
}

impl Default for Endian {
    fn default() -> Self {
        Self::Big
    }
}
