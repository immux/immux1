#[derive(Clone, Debug, PartialEq)]
pub struct KVValue(Vec<u8>);

impl KVValue {
    pub fn new(data: &[u8]) -> Self {
        Self(data.to_owned())
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for KVValue {
    fn from(data: Vec<u8>) -> KVValue {
        KVValue::new(&data)
    }
}

impl From<&[u8]> for KVValue {
    fn from(data: &[u8]) -> KVValue {
        KVValue::new(data)
    }
}

impl From<&str> for KVValue {
    fn from(data: &str) -> KVValue {
        KVValue::new(data.as_bytes())
    }
}

// Boxed

pub struct BoxedKVValue(Box<[u8]>);

impl BoxedKVValue {
    pub fn new(data: Box<[u8]>) -> Self {
        BoxedKVValue(data)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<KVValue> for BoxedKVValue {
    fn from(value: KVValue) -> Self {
        BoxedKVValue::new(value.as_bytes().to_vec().into_boxed_slice())
    }
}

#[cfg(test)]
mod kvvalue_tests {
    use crate::storage::kv::KVValue;

    #[test]
    fn test_creation() {
        let data = [1, 2, 3];
        let value = KVValue::new(&data);
        assert_eq!(value.as_bytes(), &data)
    }

    #[test]
    fn test_from_bytes() {
        let data = vec![1, 2, 3];
        let value_1 = KVValue::from(data.as_slice());
        let value_2 = KVValue::from(data);
        assert_eq!(value_1, value_2);
        assert_eq!(value_1.as_bytes(), &[1, 2, 3])
    }

    #[test]
    fn test_from_str() {
        let value = KVValue::from("aaa");
        assert_eq!(value.as_bytes(), &[97, 97, 97])
    }

}

#[cfg(test)]
mod stored_kvvalue_tests {
    use crate::storage::kv::{BoxedKVValue, KVValue};

    #[test]
    fn test_creation() {
        let data = [0u8, 1, 0];
        let value = BoxedKVValue::new(data.to_vec().into_boxed_slice());
        assert_eq!(value.as_bytes(), &data)
    }

    #[test]
    fn test_from_kvvalue() {
        let data = [0u8, 10, 20, 30];
        let kvvalue = KVValue::new(&data);
        let boxed_value = BoxedKVValue::from(kvvalue);
        assert_eq!(boxed_value.as_bytes(), &data)
    }
}
