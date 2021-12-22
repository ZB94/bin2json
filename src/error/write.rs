use deku::DekuError;

#[derive(Debug, PartialEq, Error)]
pub enum WriteBinError {
    #[error("Deku错误: {0}")]
    DekuError(#[from] DekuError),
    #[error("输入值的类型错误，需要的类型为: {0}")]
    TypeError(&'static str),
    #[error("Magic错误，需要的值为: {need:?}，输入值为: {input:?}")]
    MagicError { input: Vec<u8>, need: Vec<u8> },
    #[error("输入值超出`{0}`的有效值范围")]
    ValueOverflowOf(&'static str),
    #[error("通过引用字段大小只能存在于结构字段的定义中")]
    ByError,
    #[error("输入值的大小与定义的大小不一致")]
    BytesSizeError,
    #[error("输入数组的长度与错误，需要的长度为: {need}，输入值为: {input}")]
    LengthError { input: usize, need: usize },
}
