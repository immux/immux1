use std::collections::HashMap;

use crate::interfaces::queries::*;
use crate::storage::kv::KeyValueStore;

pub struct HashMapStore {
    pub hashmap: Option<HashMap<Vec<u8>, Vec<u8>>>,
}

impl KeyValueStore for HashMapStore {
    fn initialize(&mut self) {
        println!("Initalizing hashmap engine");
        self.hashmap = Some(HashMap::new());
    }
    fn get(&self, key: &[u8]) -> Result<QueryResponse, QueryError> {
        match &self.hashmap {
            None => Err(QueryError {
                error: String::from("HashMap: not initialized"),
            }),
            Some(hashmap) => match hashmap.get(key) {
                None => Err(QueryError {
                    error: String::from("Not found"),
                }),
                Some(value) => Ok(QueryResponse {
                    data: value.to_vec(),
                }),
            },
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<QueryResponse, QueryError> {
        match &mut self.hashmap {
            None => Err(QueryError {
                error: String::from("HashMap: not initialized"),
            }),
            Some(hashmap) => {
                match hashmap.insert(key.to_vec(), value.to_vec()) {
                    // Currently identical in both arms
                    None => Ok(QueryResponse { data: vec![] }),
                    Some(old_value) => Ok(QueryResponse { data: vec![] }),
                }
            }
        }
    }
    fn keys(&self, pattern: &str) -> Result<Vec<Vec<u8>>, QueryError> {
        match &self.hashmap {
            None => Err(QueryError {
                error: String::from("HashMap: not initialized"),
            }),
            Some(hashmap) => unimplemented!(),
        }
    }
}
