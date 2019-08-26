use serde::{Deserialize, Serialize};

use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::storage::instructions::StoreNamespace;
use crate::storage::kv::rocks::RocksEngineError;
use crate::utils::utf8_to_string;

#[derive(Debug)]
pub enum KVError {
    NotFound(KVKey),
    RocksEngine(RocksEngineError),
}

impl From<KVError> for ImmuxError {
    fn from(error: KVError) -> Self {
        ImmuxError::KV(error)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct KVKey(Vec<u8>);

impl KVKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub type KVKeySegment = KVKey;

impl From<Vec<u8>> for KVKey {
    fn from(data: Vec<u8>) -> KVKey {
        KVKey(data)
    }
}

impl From<&[u8]> for KVKey {
    fn from(data: &[u8]) -> KVKey {
        KVKey(data.to_vec())
    }
}

impl Into<Vec<u8>> for KVKey {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct KVValue(Vec<u8>);

impl KVValue {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for KVValue {
    fn from(data: Vec<u8>) -> KVValue {
        KVValue(data)
    }
}

impl From<&[u8]> for KVValue {
    fn from(data: &[u8]) -> KVValue {
        KVValue(data.to_vec())
    }
}

pub struct BoxedKVKey(Box<[u8]>);
pub struct BoxedKVValue(Box<[u8]>);

impl BoxedKVKey {
    pub fn new(data: Box<[u8]>) -> Self {
        BoxedKVKey(data)
    }
    pub fn inner(&self) -> &[u8] {
        &self.0
    }
}

impl BoxedKVValue {
    pub fn new(data: Box<[u8]>) -> Self {
        BoxedKVValue(data)
    }
    pub fn inner(&self) -> &[u8] {
        &self.0
    }
}

impl From<KVValue> for BoxedKVValue {
    fn from(value: KVValue) -> Self {
        BoxedKVValue::new(value.as_bytes().to_vec().into_boxed_slice())
    }
}

impl From<BoxedKVKey> for KVKey {
    fn from(key: BoxedKVKey) -> Self {
        KVKey(key.inner().to_vec())
    }
}

impl From<KVKey> for BoxedKVKey {
    fn from(key: KVKey) -> Self {
        BoxedKVKey(key.as_bytes().to_vec().into_boxed_slice())
    }
}

#[derive(Clone, PartialEq)]
pub struct KVNamespace(Vec<u8>);

impl KVNamespace {
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
        KVNamespace(ns.into())
    }
}

impl From<Vec<u8>> for KVNamespace {
    fn from(data: Vec<u8>) -> KVNamespace {
        KVNamespace(data)
    }
}

impl Into<Vec<u8>> for KVNamespace {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

pub trait KeyValueStore {
    fn get(&self, kvkey: &KVKey) -> ImmuxResult<KVValue>;
    fn set(&mut self, kvkey: &KVKey, value: &KVValue) -> ImmuxResult<()>;
    fn set_many(&mut self, pairs: &[(KVKey, KVValue)]) -> ImmuxResult<()>;
    fn switch_namespace(&mut self, namespace: &KVNamespace) -> ImmuxResult<()>;
    fn read_namespace(&self) -> KVNamespace;
    fn filter_prefix(&self, prefix: &KVKeySegment) -> Box<Vec<(BoxedKVKey, BoxedKVValue)>>;
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KeyValueEngine {
    HashMap,
    Rocks,
}
