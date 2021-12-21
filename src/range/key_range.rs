use std::fmt::Display;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::str::FromStr;

/// 引用键范围
///
/// **完整序列化示例(json)：**
/// ```rust
/// use bin2json::range::KeyRange;
///
/// assert_eq!(KeyRange::Value(100), serde_json::from_str::<KeyRange>(r#""100""#)?);
/// assert_eq!(KeyRange::Range(100..200), serde_json::from_str::<KeyRange>(r#""100..200""#)?);
/// assert_eq!(KeyRange::RangeFrom(100..), serde_json::from_str::<KeyRange>(r#""100..""#)?);
/// assert_eq!(KeyRange::Full, serde_json::from_str::<KeyRange>(r#""..""#)?);
/// assert_eq!(KeyRange::RangeInclusive(100..=200), serde_json::from_str::<KeyRange>(r#""100..=200""#)?);
/// assert_eq!(KeyRange::RangeTo(..200), serde_json::from_str::<KeyRange>(r#""..200""#)?);
/// assert_eq!(KeyRange::RangeToInclusive(..=200), serde_json::from_str::<KeyRange>(r#""..=200""#)?);
/// assert_eq!(KeyRange::Custom(vec![1, 2, 3]), serde_json::from_str::<KeyRange>(r#""[1, 2, 3]""#)?);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum KeyRange {
    /// 任意合法的[`i64`]
    Value(i64),
    /// `start..end`
    Range(Range<i64>),
    /// `start..`
    RangeFrom(RangeFrom<i64>),
    /// `..`
    Full,
    /// `start..=end`
    RangeInclusive(RangeInclusive<i64>),
    /// `..end`
    RangeTo(RangeTo<i64>),
    /// `..=end`
    RangeToInclusive(RangeToInclusive<i64>),
    /// `[v1, v2, ..]`
    Custom(Vec<i64>),
}

impl KeyRange {
    pub fn new<KR: Into<KeyRange>>(kr: KR) -> Self {
        kr.into()
    }

    pub fn contains(&self, value: &i64) -> bool {
        match self {
            KeyRange::Value(v) => v == value,
            KeyRange::Range(r) => r.contains(value),
            KeyRange::RangeFrom(r) => r.contains(value),
            KeyRange::Full => true,
            KeyRange::RangeInclusive(r) => r.contains(value),
            KeyRange::RangeTo(r) => r.contains(value),
            KeyRange::RangeToInclusive(r) => r.contains(value),
            KeyRange::Custom(v) => v.contains(value),
        }
    }
}

impl Display for KeyRange {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KeyRange::Value(v) => write!(fmt, "{}", v),
            KeyRange::Range(r) => write!(fmt, "{:?}", r),
            KeyRange::RangeFrom(r) => write!(fmt, "{:?}", r),
            KeyRange::Full => write!(fmt, ".."),
            KeyRange::RangeInclusive(r) => write!(fmt, "{:?}", r),
            KeyRange::RangeTo(r) => write!(fmt, "{:?}", r),
            KeyRange::RangeToInclusive(r) => write!(fmt, "{:?}", r),
            KeyRange::Custom(v) => write!(fmt, "{:?}", v),
        }
    }
}

impl FromStr for KeyRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("[") && s.ends_with("]") {
            s[1..s.len() - 1]
                .split(",")
                .map(|i| i.trim().parse::<i64>())
                .collect::<Result<Vec<_>, _>>()
                .map(|l| KeyRange::Custom(l))
                .map_err(|e| e.to_string())
        } else {
            let mut start = None;
            let mut end = None;
            let mut inclusive = false;
            let (mut left, mut right) = match s.split_once("..") {
                Some(r) => r,
                None => return s.trim().parse::<i64>()
                    .map(|v| KeyRange::new(v))
                    .map_err(|e| e.to_string()),
            };

            left = left.trim();
            if !left.is_empty() {
                start = Some(left.parse::<i64>().map_err(|e| e.to_string())?);
            }

            right = right.trim();
            if right.starts_with("=") {
                inclusive = true;
                right = right[1..].trim();
            }

            if !right.is_empty() {
                end = Some(right.parse::<i64>().map_err(|e| e.to_string())?);
            }

            let r = match (start, end, inclusive) {
                (Some(start), Some(end), false) => KeyRange::new(start..end),
                (Some(start), None, false) => KeyRange::new(start..),
                (None, None, false) => KeyRange::new(..),
                (Some(start), Some(end), true) => KeyRange::new(start..=end),
                (None, Some(end), false) => KeyRange::new(..end),
                (None, Some(end), true) => KeyRange::new(..=end),
                _ => return Err(format!("输入范围格式错误"))
            };
            Ok(r)
        }
    }
}

macro_rules! impl_kr {
    ($from: ty, $path: path) => {
        impl From<$from> for KeyRange {
            fn from(v: $from) -> Self {
                $path(v)
            }
        }
    };
}

impl_kr!(i64, KeyRange::Value);
impl_kr!(Vec<i64>, KeyRange::Custom);
impl_kr!(Range<i64>, KeyRange::Range);
impl_kr!(RangeFrom<i64>, KeyRange::RangeFrom);
impl_kr!(RangeInclusive<i64>, KeyRange::RangeInclusive);
impl_kr!(RangeTo<i64>, KeyRange::RangeTo);
impl_kr!(RangeToInclusive<i64>, KeyRange::RangeToInclusive);


impl From<&[i64]> for KeyRange {
    fn from(c: &[i64]) -> Self {
        c.to_vec().into()
    }
}

impl<const LEN: usize> From<&[i64; LEN]> for KeyRange {
    fn from(c: &[i64; LEN]) -> Self {
        c.to_vec().into()
    }
}

impl From<RangeFull> for KeyRange {
    fn from(_: RangeFull) -> Self {
        Self::Full
    }
}

impl TryFrom<String> for KeyRange {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Into<String> for KeyRange {
    fn into(self) -> String {
        self.to_string()
    }
}
