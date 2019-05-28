pub mod hashmap;
pub mod redis;
pub mod rocks;

use serde::{Deserialize, Serialize};

use crate::declarations::errors::ImmuxResult;

pub trait KeyValueStore {
    fn get(&self, key: &[u8]) -> ImmuxResult<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: &[u8]) -> ImmuxResult<Vec<u8>>;
    fn switch_namespace(&mut self, namespace: &[u8]) -> ImmuxResult<()>;
    fn read_namespace(&self) -> &[u8];
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KeyValueEngine {
    HashMap,
    Redis,
    Rocks,
}
