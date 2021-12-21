use std::collections::HashMap;
use std::fmt::Debug;

use super::KeyRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "HashMap<KeyRange, V>", from = "HashMap<KeyRange, V>")]
pub struct KeyRangeMap<V: Clone> {
    value_map: HashMap<i64, V>,
    range_map: HashMap<KeyRange, V>,
    default: Option<Box<V>>,
}

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
            KeyRange::Value(v) => self.value_map.insert(v, value),
            KeyRange::Full => self.default.replace(Box::new(value)).map(|d| *d),
            other => self.range_map.insert(other, value)
        }
    }

    pub fn get(&self, key: &i64) -> Option<&V> {
        self.value_map
            .get(key)
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
