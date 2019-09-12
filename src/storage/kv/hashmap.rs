use std::collections::HashMap;

use crate::declarations::errors::ImmuxResult;
use crate::storage::kv::{
    BoxedKVKey, BoxedKVValue, KVKey, KVKeySegment, KVNamespace, KVValue, KeyValueStore,
};

pub struct HashmapNode {
    pub name: KVNamespace,
    pub hashmap: HashMap<KVKey, KVValue>,
}

pub struct HashMapStore {
    pub hashmaps: Vec<HashmapNode>,
    pub current_node_index: usize,
}

impl HashMapStore {
    pub fn new(namespace: &KVNamespace) -> HashMapStore {
        let hashmaps = vec![HashmapNode {
            name: namespace.to_owned(),
            hashmap: HashMap::new(),
        }];
        let store = HashMapStore {
            hashmaps,
            current_node_index: 0,
        };
        store
    }
}

impl KeyValueStore for HashMapStore {
    fn get(&self, key: &KVKey) -> ImmuxResult<Option<KVValue>> {
        let node = &self.hashmaps[self.current_node_index];
        match node.hashmap.get(key.into()) {
            None => Ok(None),
            Some(value) => Ok(Some(value.to_owned())),
        }
    }
    fn set(&mut self, key: &KVKey, value: &KVValue) -> ImmuxResult<()> {
        let hashmap = &mut self.hashmaps[self.current_node_index].hashmap;
        match hashmap.insert(key.to_owned(), value.to_owned()) {
            // Currently identical in both arms
            None => Ok(()),
            Some(_old_value) => Ok(()),
        }
    }
    fn set_many(&mut self, pairs: &[(KVKey, KVValue)]) -> ImmuxResult<()> {
        let hashmap = &mut self.hashmaps[self.current_node_index].hashmap;
        for pair in pairs {
            let (key, value) = pair;
            match hashmap.insert(key.to_owned(), value.to_owned()) {
                // Currently identical in both arms
                None => {}
                Some(_old_value) => {}
            };
        }
        Ok(())
    }
    fn switch_namespace(&mut self, namespace: &KVNamespace) -> ImmuxResult<()> {
        for (index, node) in self.hashmaps.iter().enumerate() {
            if node.name == *namespace {
                self.current_node_index = index;
                return Ok(());
            }
        }
        let new_node = HashmapNode {
            name: namespace.to_owned(),
            hashmap: HashMap::new(),
        };
        self.hashmaps.push(new_node);
        self.current_node_index = self.hashmaps.len() - 1;
        return Ok(());
    }
    fn read_namespace(&self) -> KVNamespace {
        self.hashmaps[self.current_node_index].name.clone()
    }
    fn filter_prefix(&self, prefix: &KVKeySegment) -> Box<Vec<(BoxedKVKey, BoxedKVValue)>> {
        let node = &self.hashmaps[self.current_node_index];
        let keys = node
            .hashmap
            .keys()
            .filter(|key| key.as_bytes().starts_with(prefix.as_bytes()));
        let mut result: Vec<(BoxedKVKey, BoxedKVValue)> = Vec::new();
        for key in keys {
            if let Some(value) = node.hashmap.get(key) {
                let boxed_key = BoxedKVKey::from(key.clone());
                let boxed_value = BoxedKVValue::from(value.clone());
                result.push((boxed_key, boxed_value))
            }
        }
        Box::new(result)
    }
}
