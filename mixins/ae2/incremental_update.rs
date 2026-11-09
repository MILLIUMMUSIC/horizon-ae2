// SPDX-License-Identifier: LGPL-3.0-or-later
// IncrementalUpdateHelper from AE2: per-menu full-key serials and pending changes.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SerialCollision;

/// A menu retains full server keys, not the client-visible/share-tag form.
/// Ordered sets provide a deterministic native iteration; Java HashSet's
/// cross-runtime hash order is not a stable source API contract.
pub struct Updates<K> {
    mapping: BTreeMap<K, i64>,
    reverse: BTreeMap<i64, K>,
    changes: BTreeSet<K>,
    serial: i64,
    full: bool,
}
impl<K> Default for Updates<K> {
    fn default() -> Self {
        Self {
            mapping: BTreeMap::new(),
            reverse: BTreeMap::new(),
            changes: BTreeSet::new(),
            serial: 0,
            full: true,
        }
    }
}
impl<K: Ord + Clone> Updates<K> {
    pub fn serial(&self, key: &K) -> Option<i64> {
        self.mapping.get(key).copied()
    }
    pub fn get_or_assign(&mut self, key: &K) -> Result<i64, SerialCollision> {
        if let Some(serial) = self.serial(key) {
            return Ok(serial);
        }
        self.serial = self.serial.wrapping_add(1);
        // HashBiMap.computeIfAbsent increments before rejecting a duplicate
        // value. Never silently overwrite a different key if a serial wraps.
        if self.reverse.contains_key(&self.serial) {
            return Err(SerialCollision);
        }
        self.mapping.insert(key.clone(), self.serial);
        self.reverse.insert(self.serial, key.clone());
        Ok(self.serial)
    }
    pub fn by_serial(&self, serial: i64) -> Option<&K> {
        self.reverse.get(&serial)
    }
    pub fn clear(&mut self) {
        self.changes.clear();
        self.full = true;
    }
    pub fn reset(&mut self) {
        self.clear();
        self.serial = 0;
        self.mapping.clear();
        self.reverse.clear();
    }
    pub fn add_change(&mut self, key: K) {
        // Source removes/re-adds an equal key: the pending object must be the
        // newest one, even though an existing serial retains its original key.
        self.changes.replace(key);
    }
    pub fn remove_serial(&mut self, key: &K) {
        if let Some(serial) = self.mapping.remove(key) {
            self.reverse.remove(&serial);
        }
    }
    pub fn commit(&mut self) {
        self.changes.clear();
        self.full = false;
    }
    pub fn has_changes(&self) -> bool {
        self.full || !self.changes.is_empty()
    }
    pub fn is_full(&self) -> bool {
        self.full
    }
    pub fn changes(&self) -> impl Iterator<Item = &K> {
        self.changes.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;
    #[derive(Clone, Debug)]
    struct Key {
        id: u32,
        object: u32,
    }
    impl PartialEq for Key {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
    impl Eq for Key {}
    impl PartialOrd for Key {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Key {
        fn cmp(&self, other: &Self) -> Ordering {
            self.id.cmp(&other.id)
        }
    }
    #[test]
    fn full_update_keeps_serial_keys_but_pending_change_keeps_latest_object() {
        let mut updates = Updates::default();
        let old = Key { id: 7, object: 1 };
        let new = Key { id: 7, object: 2 };
        assert!(updates.has_changes() && updates.is_full());
        let serial = updates.get_or_assign(&old).unwrap();
        assert_eq!(serial, 1);
        assert_eq!(updates.get_or_assign(&new), Ok(serial));
        updates.add_change(old.clone());
        updates.add_change(new);
        assert_eq!(updates.changes().next().unwrap().object, 2);
        assert_eq!(updates.by_serial(serial).unwrap().object, 1);
        updates.commit();
        assert!(!updates.has_changes());
        updates.clear();
        assert!(updates.is_full());
        assert_eq!(updates.serial(&old), Some(serial));
        updates.remove_serial(&old);
        assert!(updates.by_serial(serial).is_none());
        assert_eq!(updates.get_or_assign(&old), Ok(2));
        updates.reset();
        assert_eq!(updates.get_or_assign(&old), Ok(1));
    }
    #[test]
    fn removed_mapping_does_not_remove_pending_change_and_serial_wrap_cannot_replace_keys() {
        let mut updates = Updates::default();
        updates.get_or_assign(&1).unwrap();
        updates.add_change(1);
        updates.remove_serial(&1);
        assert_eq!(updates.changes().copied().collect::<Vec<_>>(), [1]);
        updates.serial = i64::MAX;
        assert_eq!(updates.get_or_assign(&2), Ok(i64::MIN));
        updates.serial = i64::MAX;
        assert_eq!(updates.get_or_assign(&3), Err(SerialCollision));
        assert_eq!(updates.by_serial(i64::MIN), Some(&2));
        assert_eq!(updates.serial(&3), None);
    }
}
