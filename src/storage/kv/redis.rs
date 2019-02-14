use redis::Commands;

use crate::interfaces::queries::*;
use crate::storage::kv::KeyValueStore;

pub struct RedisStore {
    pub redis_client: Option<redis::Client>,
    pub redis_connection: Option<redis::Connection>,
}

impl KeyValueStore for RedisStore {
    fn initialize(&mut self) {
        println!("Initalizing Redis engine");
        self.redis_connection = None;
        self.redis_client = None;
        let client = redis::Client::open("redis://127.0.0.1:7777/");
        match client {
            Err(error) => eprintln!("data store initialize error: {}", error),
            Ok(client) => {
                let connection = client.get_connection();
                self.redis_client = Some(client);
                match connection {
                    Err(error) => eprintln!("connection error: {}", error),
                    Ok(connection) => {
                        self.redis_connection = Some(connection);
                    }
                }
            }
        }
    }
    fn get(&self, key: &[u8]) -> Result<QueryResponse, QueryError> {
        let connection = &self.redis_connection;
        match connection {
            None => {
                let query_error = QueryError {
                    error: String::from("GET: Cannot get connection"),
                };
                Err(query_error)
            }
            Some(connection) => {
                let result = connection.get(key) as Result<Vec<u8>, redis::RedisError>;
                match result {
                    Err(error) => {
                        let query_error = QueryError {
                            error: String::from("GET: Redis error"),
                        };
                        Err(query_error)
                    }
                    Ok(result) => {
                        let response = QueryResponse { data: result };
                        Ok(response)
                    }
                }
            }
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<QueryResponse, QueryError> {
        let connection = &self.redis_connection;
        match connection {
            None => {
                let query_error = QueryError {
                    error: String::from("SET: Cannot get connection"),
                };
                Err(query_error)
            }
            Some(connection) => {
                let result = connection.set(key, value) as Result<String, redis::RedisError>;
                match result {
                    Err(error) => {
                        let query_error = QueryError {
                            error: String::from("SET: Redis error"),
                        };
                        Err(query_error)
                    }
                    Ok(result) => {
                        let response = QueryResponse {
                            data: result.as_bytes().to_vec(),
                        };
                        Ok(response)
                    }
                }
            }
        }
    }
    fn keys(&self, pattern: &str) -> Result<Vec<Vec<u8>>, QueryError> {
        let connection = &self.redis_connection;
        match connection {
            None => {
                let query_error = QueryError {
                    error: String::from("KEYS: Cannot get connection"),
                };
                Err(query_error)
            }
            Some(connection) => {
                let x = [109, 56];
                let result = connection.keys(pattern) as Result<Vec<Vec<u8>>, redis::RedisError>;
                match result {
                    Err(error) => {
                        let query_error = QueryError {
                            error: String::from("KEYS: Redis error"),
                        };
                        Err(query_error)
                    }
                    Ok(result) => Ok(result),
                }
            }
        }
    }
}
