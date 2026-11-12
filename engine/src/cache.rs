/*
    Copyright 2026 Andrew Medworth <github@medworth.org.uk>

    This file is part of Dots-and-Boxes Engine.

    Dots-and-Boxes Engine is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Dots-and-Boxes Engine is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with Dots-and-Boxes Engine.  If not, see <http://www.gnu.org/licenses/>.
*/
// A cache of values calculated for positions (keyed by their Zobrist hashes), which can be
// limited in size to bound its memory use. When it is full, it makes room by forgetting the
// values which took least work to calculate, so are cheapest to calculate again. Forgetting a
// value only costs time: the searches using a cache never depend on a value being kept.
use std::collections::HashMap;

pub trait CacheValue: Copy {
    // How much work calculating the value took (see work_class)
    fn work_class(&self) -> u8;
}

// A measure of the work done to calculate a value, given the number of positions newly cached
// while calculating it: roughly its logarithm (base 2), so it fits in a byte
pub fn work_class(positions: u64) -> u8 {
    (u64::BITS - positions.leading_zeros()) as u8
}

pub struct Cache<V> {
    map: HashMap<u64, V>,
    max_len: usize,
    // The capacity of the table once it first held max_len values (see needs_room)
    full_capacity: Option<usize>,
    added: u64,
}

impl<V: CacheValue> Cache<V> {
    // A cache holding at most max_len values. Its hash table never grows beyond the size
    // needed for that many, so for the best use of memory, max_len should be the most a
    // table of some size holds: 7/8 of a power of 2 (at least 8).
    pub fn new(max_len: usize) -> Cache<V> {
        assert!(max_len > 0, "A cache must be able to hold at least one value");
        Cache{ map: HashMap::new(), max_len: max_len, full_capacity: None, added: 0 }
    }

    // A cache with no limit on its size
    pub fn unbounded() -> Cache<V> {
        Cache::new(usize::MAX)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    // The number of values added to the cache since it was created: values replaced aren't
    // counted, but values forgotten and calculated again are, so this measures the work done
    pub fn added(&self) -> u64 {
        self.added
    }

    pub fn get(&self, key: u64) -> Option<V> {
        self.map.get(&key).copied()
    }

    pub fn insert(&mut self, key: u64, value: V) {
        if self.needs_room() {
            self.make_room();
        } else if self.full_capacity.is_none() && self.map.len() == self.map.capacity() && self.map.len() >= self.max_len / 16 {
            // The table needs to grow, and is already a sizeable fraction of its largest size,
            // so grow it straight to that. Growing it in steps would leave the memory of each
            // smaller table free but too small for the next, so (since WebAssembly memory never
            // shrinks) would use about twice the memory of the largest table in the end.
            self.map.reserve(self.max_len - self.map.len());
        }
        if self.map.insert(key, value).is_none() {
            self.added += 1;
            if self.full_capacity.is_none() && self.map.len() >= self.max_len {
                self.full_capacity = Some(self.map.capacity());
            }
        }
    }

    // Whether room must be made before inserting a value: if the cache is full, or if inserting
    // could grow the table. The slots of forgotten values can't be reused until the table is
    // rehashed, which happens when an insertion finds no free slot left (when its capacity,
    // which counts only the free slots, is down to its length), even if it replaces a value;
    // the table then grows instead if more than half of it would be in use.
    fn needs_room(&self) -> bool {
        let len = self.map.len();
        len >= self.max_len || self.full_capacity.is_some_and(|c| len == self.map.capacity() && (len + 1) * 2 > c)
    }

    // Forget all values, freeing their memory
    pub fn clear(&mut self) {
        self.map = HashMap::new();
    }

    // Forget more than half the values, keeping those which took most work. Leaving the table
    // at most half full means its next rehash (to reuse the slots of the values forgotten)
    // happens in place, without growing it.
    fn make_room(&mut self) {
        let mut counts = [0usize; u64::BITS as usize + 1];
        for v in self.map.values() {
            counts[v.work_class() as usize] += 1;
        }
        let mut forgetting = 0;
        let mut threshold = 0;
        while forgetting * 2 <= self.map.len() {
            forgetting += counts[threshold];
            threshold += 1;
        }
        self.map.retain(|_, v| v.work_class() as usize >= threshold);
    }

    #[cfg(test)]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl CacheValue for u8 {
        fn work_class(&self) -> u8 {
            *self
        }
    }

    #[test]
    fn work_classes() {
        assert_eq!(0, work_class(0));
        assert_eq!(1, work_class(1));
        assert_eq!(2, work_class(2));
        assert_eq!(2, work_class(3));
        assert_eq!(3, work_class(4));
        assert_eq!(64, work_class(u64::MAX));
    }

    #[test]
    fn unbounded_keeps_everything() {
        let mut cache = Cache::unbounded();
        for key in 0..10_000 {
            cache.insert(key, 0u8);
        }
        assert_eq!(10_000, cache.len());
        assert_eq!(10_000, cache.added());
        // Replacing a value isn't adding one
        cache.insert(0, 1u8);
        assert_eq!(Some(1), cache.get(0));
        assert_eq!(10_000, cache.added());
    }

    #[test]
    fn full_cache_forgets_least_work() {
        let mut cache = Cache::new(4);
        cache.insert(1, 5u8);
        cache.insert(2, 1u8);
        cache.insert(3, 4u8);
        cache.insert(4, 3u8);
        assert_eq!(4, cache.len());
        // Adding one forgets more than half, those with least work
        cache.insert(5, 0u8);
        assert_eq!(Some(5), cache.get(1));
        assert_eq!(None, cache.get(2));
        assert_eq!(None, cache.get(3));
        assert_eq!(None, cache.get(4));
        assert_eq!(Some(0), cache.get(5));
        assert_eq!(5, cache.added());
    }

    #[test]
    fn full_cache_forgets_even_if_all_equal() {
        let mut cache = Cache::new(4);
        for key in 0..5 {
            cache.insert(key, 7u8);
        }
        assert_eq!(1, cache.len());
        assert_eq!(Some(7), cache.get(4));
    }

    #[test]
    fn table_grows_straight_to_largest_size() {
        let max_len = (1 << 12) / 8 * 7;
        let mut cache = Cache::new(max_len);
        for key in 0..max_len as u64 / 16 + 1 {
            cache.insert(key, 0u8);
        }
        assert_eq!(max_len, cache.capacity());
        // Unbounded tables grow in steps as usual
        let mut cache = Cache::unbounded();
        for key in 0..1000 {
            cache.insert(key, 0u8);
        }
        assert!(cache.capacity() < 2000);
    }

    #[test]
    fn table_never_grows_once_full() {
        // Whether or not the table is exactly the size for the most values cached
        for max_len in [(1 << 12) / 8 * 7, 1000] {
            let mut cache = Cache::new(max_len);
            for key in 0..max_len as u64 {
                cache.insert(key, (key % 13) as u8);
            }
            // (The capacity reported shrinks as values are forgotten, until their slots are
            // reused, so it's at most this)
            let capacity = cache.capacity();
            for key in max_len as u64..20 * max_len as u64 {
                cache.insert(key.wrapping_mul(0x9e3779b97f4a7c15), (key % 13) as u8);
                // Replacing values, too (as searches do when they improve on a cached value)
                cache.insert(key.wrapping_mul(0x9e3779b97f4a7c15), (key % 11) as u8);
                assert!(cache.len() <= max_len);
                assert!(cache.capacity() <= capacity, "{} > {} for {}", cache.capacity(), capacity, max_len);
            }
        }
    }
}
