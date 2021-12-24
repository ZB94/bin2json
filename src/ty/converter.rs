use serde_json::Value;

use crate::ty::utils::{set_ctx, to_json_value};

/// 数值验证和转化
/// 总字节大小
///
/// **示例：**
/// ```rust
/// use bin2json::ty::Converter;
///
/// let c: Converter = serde_json::from_str(r#"{ "convert": "1 + 1" }"#)?;
/// assert_eq!(Converter::new("1 + 1"), c);
///
/// let c: Converter = serde_json::from_str(r#"{
///     "before_valid": "true",
///     "convert": "1 + 1",
///     "after_valid": "false"
/// }"#)?;
/// assert_eq!(Converter{
///     before_valid: Some("true".to_string()),
///     convert: Some("1 + 1".to_string()),
///     after_valid: Some("false".to_string()),
/// }, c);
/// # Ok::<_, serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Converter {
    /// 转化之前的验证。表达式结果应为布尔值
    #[serde(default)]
    pub before_valid: Option<String>,
    /// 转化
    #[serde(default)]
    pub convert: Option<String>,
    /// 转化之后的验证。表达式结果应为布尔值
    #[serde(default)]
    pub after_valid: Option<String>,
}

impl Converter {
    pub fn new<S: Into<String>>(convert: S) -> Self {
        Self {
            before_valid: None,
            convert: Some(convert.into()),
            after_valid: None,
        }
    }

    pub fn convert(&self, value: Value) -> evalexpr::EvalexprResult<Value> {
        let mut ctx = evalexpr::HashMapContext::new();
        set_ctx(&value, None, &mut ctx)?;

        if let Some(expr) = &self.before_valid {
            Self::valid(expr, &ctx, "转化前")?;
        }

        let value = if let Some(expr) = &self.convert {
            let v = evalexpr::eval_with_context(expr, &ctx)?;
            to_json_value(v)
        } else {
            value
        };

        if let Some(expr) = &self.after_valid {
            set_ctx(&value, None, &mut ctx)?;
            Self::valid(expr, &ctx, "转换后")?;
        }

        Ok(value)
    }

    #[inline]
    fn valid(expr: &str, ctx: &evalexpr::HashMapContext, tag: &'static str) -> evalexpr::EvalexprResult<()> {
        evalexpr::eval_boolean_with_context(expr, ctx)
            .and_then(|r| {
                if r {
                    Ok(())
                } else {
                    Err(evalexpr::EvalexprError::CustomMessage(format!("{}校验失败", tag)))
                }
            })
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self {
            before_valid: None,
            convert: None,
            after_valid: None,
        }
    }
}
