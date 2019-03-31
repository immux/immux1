pub mod hashmap;
pub mod redis;

use crate::errors::UnumError;

pub type KvResult<T> = Result<T, UnumError>;

pub trait KeyValueStore {
    fn initialize(&mut self) -> KvResult<()>;
    fn get(&self, key: &[u8]) -> KvResult<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: &[u8]) -> KvResult<Vec<u8>>;
    fn keys(&self, pattern: &str) -> KvResult<Vec<Vec<u8>>>;
}

pub enum KeyValueEngine {
    HashMap,
    Redis,
}
