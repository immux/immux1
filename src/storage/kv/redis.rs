use redis::{Commands, RedisError};

use crate::declarations::errors::UnumError::RedisEngine;
use crate::declarations::errors::{UnumError, UnumResult};
use crate::storage::kv::KeyValueStore;

#[derive(Debug)]
pub enum RedisEngineError {
    ClientInitializationError(RedisError),
    ConnectionError(RedisError),
    GetError(RedisError),
    SetError(RedisError),
}

pub struct RedisStore {
    namespace: Vec<u8>,
    pub redis_client: redis::Client,
    pub redis_connection: redis::Connection,
}

impl RedisStore {
    pub fn new(namespace: &[u8]) -> UnumResult<RedisStore> {
        let client = redis::Client::open("redis://127.0.0.1:7777/");
        match client {
            Err(error) => {
                return Err(RedisEngineError::ClientInitializationError(error).into());
            }
            Ok(client) => {
                let connection = client.get_connection();
                match connection {
                    Err(error) => Err(RedisEngineError::ConnectionError(error).into()),
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
            Err(error) => Err(RedisEngineError::GetError(error).into()),
            Ok(result) => Ok(result),
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> UnumResult<Vec<u8>> {
        let mut prefixed_key: Vec<u8> = self.namespace.to_vec();
        prefixed_key.extend_from_slice(key);
        let result =
            self.redis_connection.set(prefixed_key, value) as Result<String, redis::RedisError>;
        match result {
            Err(error) => Err(RedisEngineError::SetError(error).into()),
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
