use deku::DekuError;

#[derive(Debug, PartialEq, Error)]
pub enum WriteBinError {
    #[error("Deku错误: {0}")]
    DekuError(#[from] DekuError),
    #[error("输入值的类型错误，需要的类型为: {0}")]
    TypeError(&'static str),
    #[error("输入值超出`{0}`的有效值范围")]
    ValueOverflowOf(&'static str),
    #[error("引用字段大小只能存在于结构字段的定义中，且被引用字段定义须在引用字段之前")]
    ByError,
    #[error("枚举类型的引用字段类型错误。引用字段的类型必须是整型")]
    EnumByTypeError,
    #[error("输入值的大小与定义的大小不一致")]
    BytesSizeError,
    #[error("输入数组的长度与错误，需要的长度为: {need}，输入值为: {input}")]
    LengthError { input: usize, need: usize },
    #[error("枚举值对应零个或多个值")]
    EnumError,
    #[error("缺少字段`{0}`或输入为空")]
    MissField(String),
    #[error("表达式执行失败: {0}")]
    EvalExprError(#[from] evalexpr::EvalexprError),
    #[error("输入数据不满足校验和计算条件")]
    ChecksumError,
    #[error("加密失败: {0}")]
    EncryptError(String),
    #[error("签名失败: {0}")]
    SignError(String),
}
