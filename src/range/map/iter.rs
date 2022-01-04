use std::collections::hash_map::Iter as HashIter;

use crate::range::KeyRange;

use super::KeyRangeMap;

pub struct Iter<'a, V: Clone> {
    iter_flag: u8,
    value_iter: HashIter<'a, KeyRange, V>,
    range_iter: HashIter<'a, KeyRange, V>,
    full: Option<&'a V>,
}

impl<'a, V: Clone> Iter<'a, V> {
    pub(super) fn new(map: &'a KeyRangeMap<V>) -> Self {
        Self {
            iter_flag: 0,
            value_iter: map.value_map.iter(),
            range_iter: map.range_map.iter(),
            full: map.default.as_deref(),
        }
    }
}

impl<'a, V: Clone> Iterator for Iter<'a, V> {
    type Item = (&'a KeyRange, &'a V);

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
                .map(|v| (&KeyRange::Full, v))
        }
    }
}
