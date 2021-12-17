use std::str::Utf8Error;
use std::string::FromUtf8Error;

use deku::DekuError;

#[derive(Debug, Error, PartialEq)]
pub enum ReadBinError {
    #[error("魔法值({0:?})不对应")]
    MagicError(Vec<u8>),
    #[error("Deku错误: {0}")]
    DekuError(DekuError),
    #[error("指向的长度字段({0})的值无效")]
    LengthTargetIsInvalid(String),
    #[error("输入数据不是合法字符串: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("未能找到引用的键({0})")]
    ByKeyNotFound(String),
    #[error("未能找到以指定值结尾({0:?})的数据")]
    EndNotFound(Vec<u8>),
    #[error("输入数据不完整")]
    Incomplete,
    #[error("未能找到枚举值({0})")]
    EnumKeyNotFound(i64),
}

impl From<FromUtf8Error> for ReadBinError {
    fn from(e: FromUtf8Error) -> Self {
        e.utf8_error().into()
    }
}

impl From<DekuError> for ReadBinError {
    fn from(e: DekuError) -> Self {
        if let DekuError::Incomplete(_) = &e {
            Self::Incomplete
        } else {
            Self::DekuError(e)
        }
    }
}
