use std::collections::HashMap;

use crate::declarations::errors::{UnumError, UnumResult};
use crate::storage::kv::KeyValueStore;

pub struct HashmapNode {
    pub name: Vec<u8>,
    pub hashmap: HashMap<Vec<u8>, Vec<u8>>,
}

pub struct HashMapStore {
    pub hashmaps: Vec<HashmapNode>,
    pub current_node: usize,
}

impl HashMapStore {
    pub fn new(namespace: &[u8]) -> HashMapStore {
        let hashmaps = vec![HashmapNode {
            name: namespace.into(),
            hashmap: HashMap::new(),
        }];
        let store = HashMapStore {
            hashmaps,
            current_node: 0,
        };
        store
    }
}

impl KeyValueStore for HashMapStore {
    fn get(&self, key: &[u8]) -> UnumResult<Vec<u8>> {
        let node = &self.hashmaps[self.current_node];
        match node.hashmap.get(key) {
            None => Err(UnumError::ReadError),
            Some(value) => Ok(value.to_vec()),
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> UnumResult<Vec<u8>> {
        let hashmap = &mut self.hashmaps[self.current_node].hashmap;
        match hashmap.insert(key.to_vec(), value.to_vec()) {
            // Currently identical in both arms
            None => Ok(vec![]),
            Some(_old_value) => Ok(vec![]),
        }
    }
    fn switch_namespace(&mut self, namespace: &[u8]) -> UnumResult<()> {
        for (index, node) in self.hashmaps.iter().enumerate() {
            if node.name == namespace {
                self.current_node = index;
                return Ok(());
            }
        }
        let new_node = HashmapNode {
            name: namespace.into(),
            hashmap: HashMap::new(),
        };
        self.hashmaps.push(new_node);
        self.current_node = self.hashmaps.len() - 1;
        return Ok(());
    }
    fn read_namespace(&self) -> &[u8] {
        &self.hashmaps[self.current_node].name
    }
}
