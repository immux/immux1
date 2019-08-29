#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct KVKey(Vec<u8>);

impl KVKey {
    pub fn new(data: &[u8]) -> Self {
        Self(data.to_owned())
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub type KVKeySegment = KVKey;

impl From<Vec<u8>> for KVKey {
    fn from(data: Vec<u8>) -> KVKey {
        KVKey::new(&data)
    }
}

impl From<&[u8]> for KVKey {
    fn from(data: &[u8]) -> KVKey {
        KVKey::new(data)
    }
}

impl From<&str> for KVKey {
    fn from(data: &str) -> KVKey {
        KVKey::new(data.as_bytes())
    }
}

impl From<KVKey> for Vec<u8> {
    fn from(key: KVKey) -> Vec<u8> {
        key.0
    }
}

// Boxed

pub struct BoxedKVKey(Box<[u8]>);

impl BoxedKVKey {
    pub fn new(data: Box<[u8]>) -> Self {
        BoxedKVKey(data)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<BoxedKVKey> for KVKey {
    fn from(key: BoxedKVKey) -> Self {
        KVKey(key.as_bytes().to_vec())
    }
}

impl From<KVKey> for BoxedKVKey {
    fn from(key: KVKey) -> Self {
        BoxedKVKey(key.as_bytes().to_vec().into_boxed_slice())
    }
}

#[cfg(test)]
mod kvkey_tests {
    use super::KVKey;

    #[test]
    fn test_from_vec() {
        let input = vec![1, 2, 3];
        let key = KVKey::from(input.clone());
        assert_eq!(key.as_bytes(), input.as_slice())
    }

    #[test]
    fn test_from_slice() {
        let input = vec![3, 2, 1, 0];
        let key = KVKey::from(input.as_slice());
        assert_eq!(key.as_bytes(), input.as_slice())
    }

    #[test]
    fn test_from_str() {
        let thing = "abc";
        let key = KVKey::from(thing);
        assert_eq!(key.as_bytes(), &[97, 98, 99])
    }

    #[test]
    fn test_to_vec() {
        let key = KVKey::new(&[1, 2, 3]);
        let v: Vec<u8> = key.into();
        assert_eq!(v, vec![1, 2, 3])
    }
}

#[cfg(test)]
mod boxed_kvkey_tests {
    use crate::storage::kv::{BoxedKVKey, KVKey};

    #[test]
    fn test_creation() {
        let data: Vec<u8> = vec![1, 2, 3];
        let key = BoxedKVKey::new(data.clone().into_boxed_slice());
        let bytes = key.as_bytes();
        assert_eq!(bytes, data.as_slice())
    }

    #[test]
    fn test_from_kvkey() {
        let data = [1, 2, 3];
        let kvkey = KVKey::new(&data);
        let boxed_key = BoxedKVKey::from(kvkey);
        assert_eq!(boxed_key.as_bytes(), &data)
    }

    #[test]
    fn test_to_kvkey() {
        let data: Vec<u8> = vec![1, 2, 3];
        let boxed_key = BoxedKVKey::new(data.clone().into_boxed_slice());
        let kvkey: KVKey = boxed_key.into();
        assert_eq!(kvkey.as_bytes(), data.as_slice())
    }
}
