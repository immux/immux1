use std::collections::HashMap;

use crate::errors::UnumError;
use crate::interfaces::result::UnumResult;
use crate::storage::kv::KeyValueStore;

pub struct HashMapStore {
    pub hashmap: HashMap<Vec<u8>, Vec<u8>>,
}

impl HashMapStore {
    pub fn new() -> HashMapStore {
        let store = HashMapStore {
            hashmap: HashMap::new(),
        };
        store
    }
}

impl KeyValueStore for HashMapStore {
    fn get(&self, key: &[u8]) -> UnumResult<Vec<u8>> {
        match self.hashmap.get(key) {
            None => Err(UnumError::ReadError),
            Some(value) => Ok(value.to_vec()),
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> UnumResult<Vec<u8>> {
        match self.hashmap.insert(key.to_vec(), value.to_vec()) {
            // Currently identical in both arms
            None => Ok(vec![]),
            Some(_old_value) => Ok(vec![]),
        }
    }
}
