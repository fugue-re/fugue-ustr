use std::hash::{Hash, Hasher};

use rkyv::collections::swiss_table::{ArchivedHashMap, HashMapResolver};
use rkyv::string::{ArchivedString, StringResolver};
use rkyv::{Archive, Deserialize, Place, Serialize};

use crate::{Ustr, UstrMap};

pub type ArchivedUstrMap<V> =
    ArchivedHashMap<ArchivedString, <V as Archive>::Archived>;

impl Archive for Ustr {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedString::resolve_from_str(self.as_str(), resolver, out);
    }
}

impl<S> Serialize<S> for Ustr
where
    S: rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
        + rkyv::ser::Allocator
        + rkyv::ser::Writer
        + ?Sized,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedString::serialize_from_str(self.as_str(), serializer)
    }
}

impl<D> Deserialize<Ustr, D> for ArchivedString
where
    D: rkyv::rancor::Fallible + ?Sized,
{
    fn deserialize(&self, _: &mut D) -> Result<Ustr, D::Error> {
        Ok(Ustr::from(self.as_str()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct StrHashedUstr(Ustr);

impl Hash for StrHashedUstr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
    }
}

impl Archive for StrHashedUstr {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedString::resolve_from_str(self.0.as_str(), resolver, out);
    }
}

impl<S> Serialize<S> for StrHashedUstr
where
    S: rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
        + rkyv::ser::Allocator
        + rkyv::ser::Writer
        + ?Sized,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedString::serialize_from_str(self.0.as_str(), serializer)
    }
}

impl<V: Archive> Archive for UstrMap<V> {
    type Archived = ArchivedUstrMap<V>;
    type Resolver = HashMapResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedHashMap::resolve_from_len(self.len(), (7, 8), resolver, out);
    }
}

impl<V, S> Serialize<S> for UstrMap<V>
where
    V: Serialize<S>,
    S: rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
        + rkyv::ser::Allocator
        + rkyv::ser::Writer
        + ?Sized,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        ArchivedUstrMap::<V>::serialize_from_iter::<_, _, _, StrHashedUstr, V, S>(
            self.iter().map(|(k, v)| (StrHashedUstr(*k), v)),
            (7, 8),
            serializer,
        )
    }
}

impl<V, D> Deserialize<UstrMap<V>, D> for ArchivedUstrMap<V>
where
    V: Archive,
    V::Archived: Deserialize<V, D>,
    D: rkyv::rancor::Fallible + ?Sized,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<UstrMap<V>, D::Error> {
        let mut map = UstrMap::with_capacity(self.len());
        for (k, v) in self.iter() {
            map.insert(Ustr::from(k.as_str()), v.deserialize(deserializer)?);
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use rkyv::string::ArchivedString;

    use crate::Ustr;

    #[test]
    fn test_ustr_roundtrip() {
        let original = Ustr::from("hello rkyv");

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&original).unwrap();

        let archived =
            unsafe { rkyv::access_unchecked::<ArchivedString>(&bytes) };
        assert_eq!(archived.as_str(), "hello rkyv");

        let deserialized =
            rkyv::deserialize::<Ustr, rkyv::rancor::Error>(archived).unwrap();
        assert_eq!(deserialized, original);
        assert_eq!(deserialized.as_str(), "hello rkyv");
    }

    #[test]
    fn test_ustr_map_roundtrip() {
        use rkyv::rancor::Error;

        use crate::UstrMap;
        use super::ArchivedUstrMap;

        let mut original = UstrMap::<u32>::default();
        original.insert(Ustr::from("alpha"), 1);
        original.insert(Ustr::from("beta"), 2);
        original.insert(Ustr::from("gamma"), 3);

        let bytes = rkyv::to_bytes::<Error>(&original).unwrap();

        let archived =
            unsafe { rkyv::access_unchecked::<ArchivedUstrMap<u32>>(&bytes) };
        assert_eq!(archived.len(), 3);
        assert_eq!(archived.get("alpha").map(|v| v.to_native()), Some(1));
        assert_eq!(archived.get("beta").map(|v| v.to_native()), Some(2));
        assert_eq!(archived.get("gamma").map(|v| v.to_native()), Some(3));

        let deserialized =
            rkyv::deserialize::<UstrMap<u32>, Error>(archived).unwrap();
        assert_eq!(deserialized.len(), 3);
        assert_eq!(deserialized.get(&Ustr::from("alpha")), Some(&1));
        assert_eq!(deserialized.get(&Ustr::from("beta")), Some(&2));
        assert_eq!(deserialized.get(&Ustr::from("gamma")), Some(&3));
    }
}
