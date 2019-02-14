use std::collections::HashMap;

use crate::errors::UnumError;
use crate::storage::kv::KeyValueStore;
use crate::storage::kv::KvResult;

pub struct HashMapStore {
    pub hashmap: Option<HashMap<Vec<u8>, Vec<u8>>>,
}

impl KeyValueStore for HashMapStore {
    fn initialize(&mut self) -> KvResult<()> {
        println!("Initalizing hashmap engine");
        self.hashmap = Some(HashMap::new());
        Ok(())
    }
    fn get(&self, key: &[u8]) -> KvResult<Vec<u8>> {
        match &self.hashmap {
            None => Err(UnumError::EngineNotInitialized),
            Some(hashmap) => match hashmap.get(key) {
                None => Err(UnumError::ReadError),
                Some(value) => Ok(value.to_vec()),
            },
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> KvResult<Vec<u8>> {
        match &mut self.hashmap {
            None => Err(UnumError::EngineNotInitialized),
            Some(hashmap) => {
                match hashmap.insert(key.to_vec(), value.to_vec()) {
                    // Currently identical in both arms
                    None => Ok(vec![]),
                    Some(old_value) => Ok(vec![]),
                }
            }
        }
    }
    fn keys(&self, pattern: &str) -> KvResult<Vec<Vec<u8>>> {
        match &self.hashmap {
            None => Err(UnumError::EngineNotInitialized),
            Some(hashmap) => unimplemented!(),
        }
    }
}
