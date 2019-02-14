mod config;
mod interfaces;
mod storage;
mod utils;

use std::collections;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use crate::interfaces::http::parse_http_request;
use crate::interfaces::queries::*;
use crate::storage::kv::KeyValueStore;
use crate::storage::vkv::UnumVersionedKeyValueStore;
use crate::storage::vkv::VersionedKeyValueStore;
use crate::storage::kv::KeyValueEngine;

trait DataStore {
    fn initialize(&mut self) -> ();
    fn execute(&mut self, query: Query) -> QueryReturns;
}

pub struct UnumDB {
    store: UnumVersionedKeyValueStore,
}

impl UnumDB {
    fn new() -> UnumDB {
        let mut db = UnumDB {
//            store: UnumVersionedKeyValueStore::new(KeyValueEngine::Redis),
            store: UnumVersionedKeyValueStore::new(KeyValueEngine::HashMap),
        };
        db
    }
}

impl DataStore for UnumDB {
    fn initialize(&mut self) {
        self.store.initialize();
    }
    fn execute(&mut self, query: Query) -> Result<QueryResponse, QueryError> {
        match query {
            Query::GetKey(query) => self.store.get(&query.key),
            Query::SetKey(query) => self.store.set(&query.key, &query.value),
            Query::GetKeyAtHeight(query) => {
                self.store.get_at_version_number(&query.key, query.height)
            }
            Query::RevertAll(query) => self.store.revert_all(query.height),
            Query::RevertByKey(query) => self.store.revert_one(&query.key, query.height),
        }
    }
}

pub fn handle_connection(mut stream: TcpStream, db: &mut UnumDB) {
    let mut buffer = [0; 1024];
    let stream_reading = stream.read(&mut buffer);
    match stream_reading {
        Err(error) => {
            eprintln!("stream read errro: {:?}", error);
            return;
        }
        Ok(bytes_read) => {
            let s = String::from_utf8_lossy(&buffer[..bytes_read]);
            let query = parse_http_request(&s);
            let response = match query {
                None => {
                    let error = QueryError {
                        error: String::from("Error forming query"),
                    };
                    Err(error)
                }
                Some(query) => db.execute(query),
            };

            let mut http_response = String::from("HTTP/1.1 200 OK\r\n\r\n");

            match response {
                Err(error) => {
                    eprintln!("{:?}", error);
                    http_response += "GET ERROR";
                }
                Ok(response) => {
                    http_response += &utils::utf8_to_string(&response.data);
                }
            }
            println!("http response:\n---- \n{}\n----", http_response);
            let stream_writing = stream.write(http_response.as_bytes());
            match stream_writing {
                Err(error) => eprintln!("Stream writing error: {:?}", error),
                Ok(bytes_written) => {
                    let flushing = stream.flush();
                    match flushing {
                        Err(error) => eprintln!("Stream flushing error: {:?}", error),
                        Ok(done) => {
                            // Done
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind(config::HTTP_ENDPOINT);
    match listener {
        Err(error) => panic!(error),
        Ok(listener) => {
            let mut db = UnumDB::new();

            for stream in listener.incoming() {
                match stream {
                    Err(error) => eprintln!("Stream error {}", error),
                    Ok(stream) => handle_connection(stream, &mut db),
                }
            }
        }
    }
}
