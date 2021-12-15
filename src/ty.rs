use std::collections::HashMap;

pub use deku::ctx::{Endian, Size};

#[derive(Debug, Clone)]
pub enum Type {
    Magic(Vec<u8>),
    Boolean(Unit),
    Int8(Unit),
    Int16(Unit),
    Int32(Unit),
    Int64(Unit),
    Uint8(Unit),
    Uint16(Unit),
    Uint32(Unit),
    Uint64(Unit),
    Float32(Endian),
    Float64(Endian),
    String(Length),
    Bin(Length),
}

impl Type {
    pub fn magic(magic: &[u8]) -> Self {
        Self::Magic(magic.to_vec())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Unit {
    /// 字节顺序
    pub endian: Endian,
    /// 实际要读取的大小
    pub size: Option<Size>,
}

impl Unit {
    pub fn new(endian: Endian, size: Option<Size>) -> Self {
        Self {
            endian,
            size,
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

#[derive(Debug, Clone)]
pub enum Length {
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
        map: HashMap<isize, usize>,
    },
}

impl Length {
    pub fn by_enum<S: Into<String>>(target_field: S, map: HashMap<isize, usize>) -> Self {
        Self::Enum {
            by: target_field.into(),
            map,
        }
    }

    pub fn by_field<S: Into<String>>(target: S) -> Self {
        Self::By(target.into())
    }
}
