use redis::Commands;

use crate::errors::UnumError;
use crate::interfaces::result::UnumResult;
use crate::storage::kv::KeyValueStore;

pub struct RedisStore {
    pub redis_client: redis::Client,
    pub redis_connection: redis::Connection,
}

impl RedisStore {
    pub fn new() -> Result<RedisStore, UnumError> {
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
        match self.redis_connection.get(key) as Result<Vec<u8>, redis::RedisError> {
            Err(_error) => Err(UnumError::ReadError),
            Ok(result) => Ok(result),
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> UnumResult<Vec<u8>> {
        let result = self.redis_connection.set(key, value) as Result<String, redis::RedisError>;
        match result {
            Err(_error) => Err(UnumError::WriteError),
            Ok(result) => Ok(result.as_bytes().to_vec()),
        }
    }
}
