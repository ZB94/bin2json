use crate::Type;

/// 结构字段
///
/// 在序列化时，`ty`属性直接映射了[`Type`]的属性。如:
/// ```rust
/// use bin2json::ty::{Field, BitSize, Unit, Endian, Type};
/// let json = r#"
/// {
///     "name": "field name",
///     "type": "Uint16",
///     "unit": {
///         "endian": "Big",
///         "size": {
///             "type": "Bytes",
///             "value": 1
///         }
///     }
/// }
/// "#;
/// let field = Field::new(
///     "field name",
///     Type::Uint16 { unit: Unit::new(Endian::Big, BitSize(8)) }
/// );
/// assert_eq!(field, serde_json::from_str::<Field>(json)?);
/// # Ok::<_, serde_json::Error>(())
/// ```
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
