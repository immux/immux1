use redis::Commands;

use crate::errors::UnumError;
use crate::storage::kv::KeyValueStore;
use crate::storage::kv::KvResult;

pub struct RedisStore {
    pub redis_client: Option<redis::Client>,
    pub redis_connection: Option<redis::Connection>,
}

impl KeyValueStore for RedisStore {
    fn initialize(&mut self) -> KvResult<()> {
        println!("Initializing Redis engine");
        self.redis_connection = None;
        self.redis_client = None;
        let client = redis::Client::open("redis://127.0.0.1:7777/");
        match client {
            Err(error) => {
                eprintln!("data store initialize error: {}", error);
                return Err(UnumError::InitializationFail);
            }
            Ok(client) => {
                let connection = client.get_connection();
                self.redis_client = Some(client);
                match connection {
                    Err(error) => {
                        eprintln!("connection error: {}", error);
                        return Err(UnumError::InitializationFail);
                    }
                    Ok(connection) => {
                        self.redis_connection = Some(connection);
                        Ok(())
                    }
                }
            }
        }
    }
    fn get(&self, key: &[u8]) -> KvResult<Vec<u8>> {
        let connection = &self.redis_connection;
        match connection {
            None => Err(UnumError::EngineConnectionFail),
            Some(connection) => {
                let result = connection.get(key) as Result<Vec<u8>, redis::RedisError>;
                match result {
                    Err(error) => Err(UnumError::ReadError),
                    Ok(result) => Ok(result),
                }
            }
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> KvResult<Vec<u8>> {
        let connection = &self.redis_connection;
        match connection {
            None => Err(UnumError::EngineConnectionFail),
            Some(connection) => {
                let result = connection.set(key, value) as Result<String, redis::RedisError>;
                match result {
                    Err(error) => Err(UnumError::WriteError),
                    Ok(result) => Ok(result.as_bytes().to_vec()),
                }
            }
        }
    }
    fn keys(&self, pattern: &str) -> KvResult<Vec<Vec<u8>>> {
        let connection = &self.redis_connection;
        match connection {
            None => Err(UnumError::EngineConnectionFail),
            Some(connection) => {
                let result = connection.keys(pattern) as Result<Vec<Vec<u8>>, redis::RedisError>;
                match result {
                    Err(error) => Err(UnumError::ReadError),
                    Ok(result) => Ok(result),
                }
            }
        }
    }
}
