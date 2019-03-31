pub mod hashmap;
pub mod redis;

use crate::interfaces::result::UnumResult;

pub trait KeyValueStore {
    fn get(&self, key: &[u8]) -> UnumResult<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: &[u8]) -> UnumResult<Vec<u8>>;
}

pub enum KeyValueEngine {
    HashMap,
    Redis,
}
