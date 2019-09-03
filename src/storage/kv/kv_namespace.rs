use crate::storage::instructions::StoreNamespace;
use crate::utils::utf8_to_string;

#[derive(Clone, PartialEq, Debug)]
pub struct KVNamespace(Vec<u8>);

impl KVNamespace {
    pub fn new(data: &[u8]) -> Self {
        Self(data.to_vec())
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl ToString for KVNamespace {
    fn to_string(&self) -> String {
        utf8_to_string(self.as_bytes())
    }
}

impl From<StoreNamespace> for KVNamespace {
    fn from(ns: StoreNamespace) -> KVNamespace {
        KVNamespace::new(ns.as_bytes())
    }
}

impl From<&[u8]> for KVNamespace {
    fn from(data: &[u8]) -> KVNamespace {
        KVNamespace::new(data)
    }
}

impl From<Vec<u8>> for KVNamespace {
    fn from(data: Vec<u8>) -> KVNamespace {
        KVNamespace::new(&data)
    }
}

impl From<&str> for KVNamespace {
    fn from(data: &str) -> KVNamespace {
        KVNamespace::new(data.as_bytes())
    }
}

impl From<KVNamespace> for Vec<u8> {
    fn from(ns: KVNamespace) -> Vec<u8> {
        ns.as_bytes().to_vec()
    }
}

#[cfg(test)]
mod kv_namespace_tests {
    use crate::storage::instructions::StoreNamespace;
    use crate::storage::kv::KVNamespace;

    #[test]
    fn test_creation() {
        let data = [1u8, 0, 1];
        let ns = KVNamespace::new(&data);
        assert_eq!(ns.as_bytes(), &data)
    }

    #[test]
    fn test_to_string() {
        let data = [97, 98, 99];
        let ns = KVNamespace::new(&data);
        assert_eq!(ns.to_string(), "abc")
    }

    #[test]
    fn test_from_store_namespace() {
        let data = [0u8, 1, 0, 1];
        let store_ns = StoreNamespace::new(&data);
        let kv_ns: KVNamespace = store_ns.into();
        assert_eq!(kv_ns.as_bytes(), &data)
    }

    #[test]
    fn test_from_bytes() {
        let data = vec![1u8, 2, 3, 4, 5];
        let ns_1 = KVNamespace::from(data.clone());
        let ns_2 = KVNamespace::from(data.as_slice());
        assert_eq!(ns_1, ns_2);
        assert_eq!(ns_1.as_bytes(), data.as_slice());
        assert_eq!(ns_2.as_bytes(), data.as_slice());
    }

    #[test]
    fn test_from_str() {
        let ns = KVNamespace::from("aaaaa");
        assert_eq!(ns.as_bytes(), &[97, 97, 97, 97, 97])
    }

    #[test]
    fn test_to_bytes() {
        let data = [0u8, 1, 2];
        let ns = KVNamespace::from(data.to_vec());
        let v: Vec<_> = ns.into();
        assert_eq!(data, v.as_slice())
    }
}
