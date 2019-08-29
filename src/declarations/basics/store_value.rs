use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StoreValue(Vec<u8>);

impl StoreValue {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl AsRef<[u8]> for StoreValue {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for StoreValue {
    fn from(data: &[u8]) -> Self {
        StoreValue(data.to_vec())
    }
}

impl From<Vec<u8>> for StoreValue {
    fn from(data: Vec<u8>) -> Self {
        StoreValue(data)
    }
}

impl From<StoreValue> for Vec<u8> {
    fn from(value: StoreValue) -> Self {
        value.0
    }
}

impl From<&StoreValue> for Vec<u8> {
    fn from(value: &StoreValue) -> Self {
        value.0.to_vec()
    }
}

impl From<StoreValue> for BoxedStoreValue {
    fn from(value: StoreValue) -> Self {
        BoxedStoreValue::new(value.as_bytes().to_vec())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoxedStoreValue(Box<[u8]>);

impl BoxedStoreValue {
    pub fn new(data: Vec<u8>) -> Self {
        BoxedStoreValue(data.into_boxed_slice())
    }
    pub fn inner(&self) -> &[u8] {
        &self.0
    }
}
