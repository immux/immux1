use redis::Commands;

use crate::declarations::errors::{UnumError, UnumResult};
use crate::storage::kv::KeyValueStore;

pub struct RedisStore {
    namespace: Vec<u8>,
    pub redis_client: redis::Client,
    pub redis_connection: redis::Connection,
}

impl RedisStore {
    pub fn new(namespace: &[u8]) -> Result<RedisStore, UnumError> {
        let client = redis::Client::open("redis://127.0.0.1:7777/");
        match client {
            Err(_error) => {
                return Err(UnumError::InitializationFail);
            }
            Ok(client) => {
                let connection = client.get_connection();
                match connection {
                    Err(_error) => Err(UnumError::InitializationFail),
                    Ok(connection) => {
                        let store = RedisStore {
                            namespace: namespace.into(),
                            redis_client: client,
                            redis_connection: connection,
                        };
                        Ok(store)
                    }
                }
            }
        }
    }
}

impl KeyValueStore for RedisStore {
    fn get(&self, key: &[u8]) -> UnumResult<Vec<u8>> {
        let mut prefixed_key: Vec<u8> = self.namespace.to_vec();
        prefixed_key.extend_from_slice(key);
        match self.redis_connection.get(prefixed_key) as Result<Vec<u8>, redis::RedisError> {
            Err(_error) => Err(UnumError::ReadError),
            Ok(result) => Ok(result),
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> UnumResult<Vec<u8>> {
        let mut prefixed_key: Vec<u8> = self.namespace.to_vec();
        prefixed_key.extend_from_slice(key);
        let result =
            self.redis_connection.set(prefixed_key, value) as Result<String, redis::RedisError>;
        match result {
            Err(_error) => Err(UnumError::WriteError),
            Ok(result) => Ok(result.as_bytes().to_vec()),
        }
    }
    fn switch_namespace(&mut self, namespace: &[u8]) -> UnumResult<()> {
        self.namespace = namespace.to_vec();
        Ok(())
    }
    fn read_namespace(&self) -> &[u8] {
        &self.namespace
    }
}
