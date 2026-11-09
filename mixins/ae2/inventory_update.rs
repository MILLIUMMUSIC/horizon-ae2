// SPDX-License-Identifier: LGPL-3.0-or-later
// MEInventoryUpdatePacket.Builder, GridInventoryEntry and Repo update rules.
// Key codecs and native per-window transport are supplied by the binding.
use super::ae2_incremental_update::{SerialCollision, Updates};
use super::ae2_me_interaction::{DecodeError, ErrorKind, read_var, write_var};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry<K> {
    pub serial: i64,
    pub key: Option<K>,
    pub stored: i64,
    pub requestable: i64,
    pub craftable: bool,
}
impl<K> Entry<K> {
    pub fn meaningful(&self) -> bool {
        self.stored > 0 || self.requestable > 0 || self.craftable
    }
}

/// The full-key codec must implement AEKey.write/readOptionalKey, including
/// its own registry/component constraints. Partial writes/reads stay visible
/// on error, just as the source buffer does. This trait invents no key format.
pub trait KeyCodec<K> {
    type Error;
    fn write(&mut self, key: Option<&K>, out: &mut Vec<u8>) -> Result<(), Self::Error>;
    fn read(&mut self, input: &[u8], at: &mut usize) -> Result<Option<K>, Self::Error>;
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    Buffer(DecodeError),
    Key(E),
    NegativeCount,
    SerialCollision,
}
impl<E> From<DecodeError> for Error<E> {
    fn from(value: DecodeError) -> Self {
        Self::Buffer(value)
    }
}
impl<E> From<SerialCollision> for Error<E> {
    fn from(_: SerialCollision) -> Self {
        Self::SerialCollision
    }
}

fn boolean<E>(input: &[u8], at: &mut usize) -> Result<bool, Error<E>> {
    let value = *input.get(*at).ok_or(DecodeError {
        kind: ErrorKind::Truncated,
        consumed: *at,
    })?;
    *at += 1;
    Ok(value != 0)
}
pub fn write_entry<K, C: KeyCodec<K>>(
    entry: &Entry<K>,
    codec: &mut C,
    out: &mut Vec<u8>,
) -> Result<(), Error<C::Error>> {
    write_var(entry.serial as u64, out);
    codec.write(entry.key.as_ref(), out).map_err(Error::Key)?;
    write_var(entry.stored as u64, out);
    write_var(entry.requestable as u64, out);
    out.push(u8::from(entry.craftable));
    Ok(())
}
pub fn read_entry<K, C: KeyCodec<K>>(
    input: &[u8],
    at: &mut usize,
    codec: &mut C,
) -> Result<Entry<K>, Error<C::Error>> {
    Ok(Entry {
        serial: read_var(input, at, 64)? as i64,
        key: codec.read(input, at).map_err(Error::Key)?,
        stored: read_var(input, at, 64)? as i64,
        requestable: read_var(input, at, 64)? as i64,
        craftable: boolean(input, at)?,
    })
}

/// A built source packet owns preencoded entries. Decoding is a distinct
/// operation; the original decoded record cannot be written without a builder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedPacket {
    pub container_id: i32,
    pub full: bool,
    pub count: i32,
    pub entries: Vec<u8>,
}
impl EncodedPacket {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_var(u64::from(self.container_id as u32), &mut out);
        out.push(u8::from(self.full));
        write_var(u64::from(self.count as u32), &mut out);
        if self.count > 0 {
            out.extend_from_slice(&self.entries);
        }
        out
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedPacket<K> {
    pub container_id: i32,
    pub full: bool,
    pub entries: Vec<Entry<K>>,
}
impl<K> DecodedPacket<K> {
    /// Like the original buffer decoder, return the consumed prefix. The
    /// native transport must enforce its own authenticated frame/size limits.
    pub fn decode<C: KeyCodec<K>>(
        input: &[u8],
        at: &mut usize,
        codec: &mut C,
    ) -> Result<Self, Error<C::Error>> {
        let container_id = read_var(input, at, 32)? as i32;
        let full = boolean(input, at)?;
        let count = read_var(input, at, 32)? as i32;
        if count < 0 {
            return Err(Error::NegativeCount);
        }
        // Do not preallocate an untrusted signed count; decode has identical
        // field consumption without the source's potentially huge allocation.
        let mut entries = Vec::new();
        for _ in 0..count {
            entries.push(read_entry(input, at, codec)?);
        }
        Ok(Self {
            container_id,
            full,
            entries,
        })
    }
}

pub struct Builder {
    container_id: i32,
    full: bool,
    entries: Option<Vec<u8>>,
    count: i32,
    packets: Vec<EncodedPacket>,
}
impl Builder {
    pub fn new(container_id: i32, full: bool) -> Self {
        Self {
            container_id,
            full,
            entries: None,
            count: 0,
            packets: Vec::new(),
        }
    }
    pub fn add<K, C: KeyCodec<K>>(
        &mut self,
        entry: &Entry<K>,
        codec: &mut C,
    ) -> Result<(), Error<C::Error>> {
        let entries = self.entries.get_or_insert_with(|| Vec::with_capacity(2048));
        write_entry(entry, codec, entries)?;
        self.count += 1;
        if entries.len() >= 512 * 1024 || self.count >= i32::from(i16::MAX) {
            self.flush();
        }
        Ok(())
    }
    fn flush(&mut self) {
        if let Some(entries) = self.entries.take() {
            self.packets.push(EncodedPacket {
                container_id: self.container_id,
                full: self.full,
                count: self.count,
                entries,
            });
            self.count = 0;
            self.full = false;
        }
    }
    /// As in source build(), repeated calls return the retained packet list.
    /// No writes means no packet, even when full=true.
    pub fn build(&mut self) -> &[EncodedPacket] {
        self.flush();
        &self.packets
    }
    pub fn add_full<K: Ord + Clone, C: KeyCodec<K>>(
        &mut self,
        updates: &mut Updates<K>,
        storage: &BTreeMap<K, i64>,
        craftable: &BTreeSet<K>,
        requestable: &BTreeMap<K, i64>,
        mut visible: impl FnMut(&K) -> bool,
        codec: &mut C,
    ) -> Result<(), Error<C::Error>> {
        let mut keys = BTreeSet::new();
        for key in storage
            .keys()
            .chain(craftable.iter())
            .chain(requestable.keys())
        {
            // HashSet.addAll keeps the first equal object (storage, then
            // craftable, then requestable). Bulk sorted collection can retain
            // a later equal key, changing its opaque codec/callback identity.
            keys.insert(key.clone());
        }
        for key in keys {
            if visible(&key) {
                let serial = updates.get_or_assign(&key)?;
                self.add(
                    &Entry {
                        serial,
                        stored: storage.get(&key).copied().unwrap_or(0),
                        requestable: requestable.get(&key).copied().unwrap_or(0),
                        craftable: craftable.contains(&key),
                        key: Some(key),
                    },
                    codec,
                )?;
            }
        }
        Ok(())
    }
    pub fn add_changes<K: Ord + Clone, C: KeyCodec<K>>(
        &mut self,
        updates: &mut Updates<K>,
        storage: &BTreeMap<K, i64>,
        craftable: &BTreeSet<K>,
        requestable: &BTreeMap<K, i64>,
        mut visible: impl FnMut(&K) -> bool,
        codec: &mut C,
    ) -> Result<(), Error<C::Error>> {
        // Source mutates serials, not the pending set, during iteration.
        for key in updates.changes().cloned().collect::<Vec<_>>() {
            if !visible(&key) {
                continue;
            }
            let (serial, send_key) = if let Some(serial) = updates.serial(&key) {
                (serial, None)
            } else {
                (updates.get_or_assign(&key)?, Some(key.clone()))
            };
            let mut entry = Entry {
                serial,
                key: send_key,
                stored: storage.get(&key).copied().unwrap_or(0),
                requestable: requestable.get(&key).copied().unwrap_or(0),
                craftable: craftable.contains(&key),
            };
            if !entry.meaningful() {
                entry.stored = 0;
                entry.requestable = 0;
                entry.craftable = false;
                self.add(&entry, codec)?;
                updates.remove_serial(&key);
            } else {
                self.add(&entry, codec)?;
            }
        }
        updates.commit();
        Ok(())
    }
}

/// Repo's serial-indexed entry mutations, separate from its search/sort/pinned
/// view and callbacks. The binding must clear/rebuild those views on full update.
pub struct Repo<K> {
    entries: BTreeMap<i64, Entry<K>>,
}
impl<K> Default for Repo<K> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}
impl<K: Clone> Repo<K> {
    pub fn entries(&self) -> &BTreeMap<i64, Entry<K>> {
        &self.entries
    }
    pub fn update(&mut self, full: bool, entries: &[Entry<K>], mut unknown: impl FnMut(i64)) {
        if full {
            self.entries.clear();
        }
        for entry in entries {
            let local = self.entries.get(&entry.serial);
            if local.is_none() && entry.key.is_none() {
                unknown(entry.serial);
                continue;
            }
            if !entry.meaningful() {
                self.entries.remove(&entry.serial);
            } else {
                let mut next = entry.clone();
                if next.key.is_none() {
                    next.key = local.and_then(|e| e.key.clone());
                }
                self.entries.insert(entry.serial, next);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[derive(Clone, Debug)]
    struct Key {
        id: i32,
        object: u8,
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
    struct Objects;
    impl KeyCodec<Key> for Objects {
        type Error = ();
        fn write(&mut self, key: Option<&Key>, out: &mut Vec<u8>) -> Result<(), ()> {
            out.push(key.map_or(0, |k| k.object));
            Ok(())
        }
        fn read(&mut self, _: &[u8], _: &mut usize) -> Result<Option<Key>, ()> {
            unreachable!()
        }
    }
    #[test]
    fn full_union_keeps_storage_key_object_and_does_not_commit_pending_changes() {
        let make = |object| Key { id: 4, object };
        let mut updates = Updates::default();
        updates.add_change(make(9));
        let mut builder = Builder::new(3, true);
        builder
            .add_full(
                &mut updates,
                &BTreeMap::from([(make(1), 5)]),
                &BTreeSet::from([make(2)]),
                &BTreeMap::from([(make(3), 8)]),
                |_| true,
                &mut Objects,
            )
            .unwrap();
        assert_eq!(builder.build()[0].entries, [1, 1, 5, 8, 1]);
        assert_eq!(updates.by_serial(1).unwrap().object, 1);
        assert!(updates.is_full());
        assert_eq!(updates.changes().next().unwrap().object, 9);
    }
    #[test]
    fn client_keeps_distinct_serials_for_equal_share_keys_and_refuses_unknown_delta() {
        let make = |serial, key, stored| Entry {
            serial,
            key,
            stored,
            requestable: 0,
            craftable: false,
        };
        let mut repo = Repo::default();
        let mut warnings = Vec::new();
        repo.update(
            true,
            &[make(1, Some("same"), 5), make(2, Some("same"), 7)],
            |_| {},
        );
        repo.update(false, &[make(1, None, 3), make(8, None, 9)], |s| {
            warnings.push(s)
        });
        assert_eq!(repo.entries()[&1].key, Some("same"));
        assert_eq!(repo.entries()[&2].stored, 7);
        assert_eq!(warnings, [8]);
        repo.update(false, &[make(1, None, 0)], |_| {});
        assert!(!repo.entries().contains_key(&1));
        repo.update(true, &[], |_| {});
        assert!(repo.entries().is_empty());
    }
}
