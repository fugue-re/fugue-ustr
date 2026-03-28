use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::ops::{Deref, DerefMut};

use byteorder::{ByteOrder, NativeEndian};

use super::Ustr;

/// A standard `HashMap` using `Ustr` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct UstrMap<V>(HashMap<Ustr, V, BuildHasherDefault<IdentityHasher>>);

impl<V> UstrMap<V> {
    pub fn new() -> Self {
        Self(HashMap::default())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity_and_hasher(
            capacity,
            BuildHasherDefault::default(),
        ))
    }
}

impl<V> Default for UstrMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> Deref for UstrMap<V> {
    type Target = HashMap<Ustr, V, BuildHasherDefault<IdentityHasher>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for UstrMap<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V> From<HashMap<Ustr, V, BuildHasherDefault<IdentityHasher>>> for UstrMap<V> {
    fn from(map: HashMap<Ustr, V, BuildHasherDefault<IdentityHasher>>) -> Self {
        Self(map)
    }
}

impl<V> From<UstrMap<V>> for HashMap<Ustr, V, BuildHasherDefault<IdentityHasher>> {
    fn from(map: UstrMap<V>) -> Self {
        map.0
    }
}

impl<V> FromIterator<(Ustr, V)> for UstrMap<V> {
    fn from_iter<I: IntoIterator<Item = (Ustr, V)>>(iter: I) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

impl<V> IntoIterator for UstrMap<V> {
    type Item = (Ustr, V);
    type IntoIter =
        std::collections::hash_map::IntoIter<Ustr, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, V> IntoIterator for &'a UstrMap<V> {
    type Item = (&'a Ustr, &'a V);
    type IntoIter =
        std::collections::hash_map::Iter<'a, Ustr, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, V> IntoIterator for &'a mut UstrMap<V> {
    type Item = (&'a Ustr, &'a mut V);
    type IntoIter =
        std::collections::hash_map::IterMut<'a, Ustr, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<V> Extend<(Ustr, V)> for UstrMap<V> {
    fn extend<I: IntoIterator<Item = (Ustr, V)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

/// A standard `HashSet` using `Ustr` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type UstrSet = HashSet<Ustr, BuildHasherDefault<IdentityHasher>>;

/// The worst hasher in the world -- the identity hasher.
#[doc(hidden)]
#[derive(Default)]
pub struct IdentityHasher {
    hash: u64,
}

impl Hasher for IdentityHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() == 8 {
            self.hash = NativeEndian::read_u64(bytes);
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

#[test]
fn test_hashing() {
    let _t = super::TEST_LOCK.lock();
    use crate::ustr as u;

    use std::hash::Hash;
    let u1 = u("the quick brown fox");
    let u2 = u("jumped over the lazy dog");

    let mut hasher = IdentityHasher::default();
    u1.hash(&mut hasher);
    assert_eq!(hasher.finish(), u1.precomputed_hash());

    let mut hasher = IdentityHasher::default();
    u2.hash(&mut hasher);
    assert_eq!(hasher.finish(), u2.precomputed_hash());

    let mut hm = UstrMap::<u32>::default();
    hm.insert(u1, 17);
    hm.insert(u2, 42);

    assert_eq!(hm.get(&u1), Some(&17));
    assert_eq!(hm.get(&u2), Some(&42));
}
