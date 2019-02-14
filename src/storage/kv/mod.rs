pub mod hashmap;
pub mod redis;
use crate::interfaces::queries::*;

pub trait KeyValueStore {
    fn initialize(&mut self) -> ();
    fn get(&self, key: &[u8]) -> Result<QueryResponse, QueryError>;
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<QueryResponse, QueryError>;
    fn keys(&self, pattern: &str) -> Result<Vec<Vec<u8>>, QueryError>;
}

pub enum KeyValueEngine {
    HashMap,
    Redis,
}
