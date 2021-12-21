use crate::Type;

/// 结构字段
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Field {
    /// 字段名称
    pub name: String,
    /// 字段类型
    #[serde(flatten)]
    pub ty: Type,
}

impl Field {
    pub fn new<S: Into<String>>(name: S, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}
