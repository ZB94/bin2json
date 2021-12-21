/// 数组长度
///
/// **示例：**
/// ```rust
/// use bin2json::ty::Length;
///
/// let length: Length = serde_json::from_str(r#"100"#)?;
/// assert_eq!(Length::Fixed(100), length);
///
/// let length: Length = serde_json::from_str(r#""field_name""#)?;
/// assert_eq!(Length::By("field_name".to_string()), length);
/// # Ok::<_, serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Length {
    /// 固定长度
    Fixed(usize),
    /// 通过指定字段指定。*使用该枚举时数据的定义应包含在结构体中，且指定的字段顺序应在数组之前*
    By(String),
}

impl Length {
    pub fn by_field<S: Into<String>>(field: S) -> Self {
        Self::By(field.into())
    }
}
