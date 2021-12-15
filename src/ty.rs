use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Type {
    Magic(Vec<u8>),
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
    Boolean(Unit),
    String(Length),
    Bin(Length),
}

#[derive(Debug, Copy, Clone)]
pub struct Unit {
    /// 字节顺序
    pub endian: Endian,
    /// 实际要读取的大小
    pub size: Option<Size>,
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
    /// 固定长度
    Fixed(usize),
    /// 通过指定字段的值。指定字段的类型必须为整数
    By(String),
    /// 以指定数据结尾
    EndWith(Vec<u8>),
    /// 根据指定字段的值有不同的大小，指定字段的类型必须为整数
    Enum {
        /// 字段名称
        by: String,
        /// 键为指定字段的值，值为大小
        map: HashMap<isize, usize>,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Debug, Copy, Clone)]
pub struct Size(usize);

impl Size {
    pub fn bits(size: usize) -> Self {
        Self(size)
    }

    pub fn bytes(size: usize) -> Self {
        Self(size * u8::BITS as usize)
    }
}
