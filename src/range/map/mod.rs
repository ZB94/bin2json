use std::collections::HashMap;
use std::fmt::Debug;

pub use into_iter::IntoIter;
pub use iter::Iter;

use super::KeyRange;

mod iter;
mod into_iter;

/// 快速创建[`KeyRangeMap`]
///
/// **示例:**
/// ```rust
/// use bin2json::range::KeyRangeMap;
/// use bin2json::range_map;
///
/// let map = KeyRangeMap::from([
///     (1.into(), 2),
///     ((3..).into(), 4),
///     ((..).into(), 5)
/// ]);
/// assert_eq!(range_map!(1 => 2, 3.. => 4, .. => 5), map);
///
/// let json = r#"
/// {
///     "1": 2,
///     "3..": 4,
///     "..": 5
/// }
/// "#;
/// assert_eq!(serde_json::from_str::<KeyRangeMap<i32>>(json)?, map);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "HashMap<KeyRange, V>", from = "HashMap<KeyRange, V>")]
pub struct KeyRangeMap<V: Clone> {
    value_map: HashMap<KeyRange, V>,
    range_map: HashMap<KeyRange, V>,
    default: Option<Box<V>>,
}

/// 快速创建[`KeyRangeMap`]
///
/// **示例:**
/// ```ignore
/// range_map! {
///     range1 => value1,
///     range2 => value2,
///     ..
/// }
/// ```
#[macro_export]
macro_rules! range_map {
    ($($key: expr => $value: expr),*) => {{
        $crate::range::KeyRangeMap::from([
            $(($key.into(), $value),)*
        ])
    }};
}

impl<V: Clone> KeyRangeMap<V> {
    pub fn new() -> Self {
        Self {
            value_map: Default::default(),
            range_map: Default::default(),
            default: None,
        }
    }

    pub fn insert<KR: Into<KeyRange>>(&mut self, range: KR, value: V) -> Option<V> {
        match range.into() {
            KeyRange::Value(v) => self.value_map.insert(v.into(), value),
            KeyRange::Full => self.default.replace(Box::new(value)).map(|d| *d),
            other => self.range_map.insert(other, value)
        }
    }

    pub fn remove(&mut self, range: KeyRange) -> Option<V> {
        match range {
            KeyRange::Value(v) => self.value_map.remove(&v.into()),
            KeyRange::Full => self.default.take().map(|d| *d),
            other => self.range_map.remove(&other)
        }
    }

    pub fn clear(&mut self) {
        self.value_map.clear();
        self.range_map.clear();
        self.default = None;
    }

    pub fn retain<F: FnMut(&KeyRange, &mut V) -> bool>(&mut self, mut f: F) {
        self.value_map.retain(|k, v| f(k, v));
        self.range_map.retain(|k, v| f(k, v));
        if let Some(full) = self.default.as_deref_mut() {
            if !f(&KeyRange::Full, full) {
                self.default = None;
            }
        }
    }

    pub fn get(&self, key: &i64) -> Option<&V> {
        self.value_map
            .get(&(*key).into())
            .or_else(|| {
                self.range_map.iter()
                    .find_map(|(k, v)| {
                        if k.contains(key) {
                            Some(v)
                        } else {
                            None
                        }
                    })
            })
            .or(self.default.as_ref().map(|d| d.as_ref()))
    }

    pub fn iter(&self) -> impl Iterator<Item=(&KeyRange, &V)> {
        Iter::new(self)
    }

    pub fn into_iter(self) -> impl Iterator<Item=(KeyRange, V)> {
        IntoIter::new(self)
    }
}

impl<V: PartialEq + Clone> KeyRangeMap<V> {
    pub fn find_key(&self, value: &V) -> Option<KeyRange> {
        self.value_map
            .iter()
            .find_map(|(k, v)| {
                if v == value {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .or_else(|| {
                self.range_map
                    .iter()
                    .find_map(|(k, v)| {
                        if v == value {
                            Some(k.clone())
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| {
                if self.default.is_some() {
                    Some(KeyRange::Full)
                } else {
                    None
                }
            })
    }
}

impl<V: PartialEq + Clone> PartialEq for KeyRangeMap<V> {
    fn eq(&self, other: &Self) -> bool {
        self.value_map == other.value_map &&
            self.range_map == other.range_map &&
            self.default == other.default
    }
}

impl<V: PartialEq + Eq + Clone> Eq for KeyRangeMap<V> {}

impl<V: Clone, I: IntoIterator<Item=(KeyRange, V)>> From<I> for KeyRangeMap<V> {
    fn from(iter: I) -> Self {
        let mut map = KeyRangeMap::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

impl<V: Clone> Into<HashMap<KeyRange, V>> for KeyRangeMap<V> {
    fn into(self) -> HashMap<KeyRange, V> {
        let mut map = self.range_map;
        for (k, v) in self.value_map {
            map.insert(k.into(), v);
        }
        if let Some(v) = self.default {
            map.insert(KeyRange::Full, *v);
        }
        map
    }
}
