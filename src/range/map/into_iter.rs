use std::collections::hash_map::IntoIter as MapIntoIter;

use crate::range::KeyRange;

use super::KeyRangeMap;

pub struct IntoIter<V: Clone> {
    iter_flag: u8,
    value_iter: MapIntoIter<KeyRange, V>,
    range_iter: MapIntoIter<KeyRange, V>,
    full: Option<V>,
}

impl<V: Clone> IntoIter<V> {
    pub(super) fn new(map: KeyRangeMap<V>) -> Self {
        Self {
            iter_flag: 0,
            value_iter: map.value_map.into_iter(),
            range_iter: map.range_map.into_iter(),
            full: map.default.map(|v| *v),
        }
    }
}

impl<V: Clone> Iterator for IntoIter<V> {
    type Item = (KeyRange, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_flag == 0 {
            let next = self.value_iter.next();
            if next.is_none() {
                self.iter_flag += 1;
                self.next()
            } else {
                next
            }
        } else if self.iter_flag == 1 {
            let next = self.range_iter.next();
            if next.is_none() {
                self.iter_flag += 1;
                self.next()
            } else {
                next
            }
        } else {
            self.full
                .take()
                .map(|v| (KeyRange::Full, v))
        }
    }
}
