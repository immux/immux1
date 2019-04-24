pub mod hashmap;
pub mod redis;
pub mod rocks;

use serde::{Deserialize, Serialize};

use crate::declarations::errors::UnumResult;

pub trait KeyValueStore {
    fn get(&self, key: &[u8]) -> UnumResult<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: &[u8]) -> UnumResult<Vec<u8>>;
    fn switch_namespace(&mut self, namespace: &[u8]) -> UnumResult<()>;
    fn read_namespace(&self) -> &[u8];
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KeyValueEngine {
    HashMap,
    Redis,
    Rocks,
}
