use std::str::Utf8Error;
use std::string::FromUtf8Error;

use deku::DekuError;


#[derive(Debug, Error)]
pub enum ParseError {
    #[error("魔法值不对应")]
    MagicError,
    #[error("Deku错误: {0}")]
    DekuError(DekuError),
    #[error("指向的长度字段的值无效")]
    LengthTargetIsInvalid,
    #[error("输入数据不是合法字符串: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("未能找到引用的键")]
    ByKeyNotFound,
    #[error("未能找到以指定值结尾的数据")]
    EndNotFound,
    #[error("输入数据不完整")]
    Incomplete,
    #[error("未能找到枚举值")]
    EnumKeyNotFound,
}


impl From<FromUtf8Error> for ParseError {
    fn from(e: FromUtf8Error) -> Self {
        e.utf8_error().into()
    }
}

impl From<DekuError> for ParseError {
    fn from(e: DekuError) -> Self {
        if let DekuError::Incomplete(_) = &e {
            Self::Incomplete
        } else {
            Self::DekuError(e)
        }
    }
}
