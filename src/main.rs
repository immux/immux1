mod config;
mod errors;
mod interfaces;
mod storage;
mod utils;

use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::errors::explain_error;
use crate::interfaces::http::parse_http_request;
use crate::interfaces::instructions::Answer;
use crate::storage::core::{CoreStore, UnumCore};
use crate::storage::kv::KeyValueEngine;

pub fn handle_connection(mut stream: TcpStream, core: &mut CoreStore) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Err(error) => {
            eprintln!("stream read error: {:?}", error);
            return;
        }
        Ok(bytes_read) => {
            let mut http_response = String::from("HTTP/1.1 200 OK\r\n\r\n");

            let request_string = String::from_utf8_lossy(&buffer[..bytes_read]);

            match parse_http_request(&request_string) {
                Err(_error) => {
                    http_response += "request parsing error";
                }
                Ok(instruction) => match core.execute(&instruction) {
                    Err(error) => {
                        http_response += "instruction execution error";
                        http_response += explain_error(error)
                    }
                    Ok(answer) => match answer {
                        Answer::GetOk(answer) => {
                            for item in answer.items {
                                http_response += &utils::utf8_to_string(&item)
                            }
                        }
                        _ => http_response += "success",
                    },
                },
            };

            match stream.write(http_response.as_bytes()) {
                Err(error) => eprintln!("Stream writing error: {:?}", error),
                Ok(_bytes_written) => {
                    let flushing = stream.flush();
                    match flushing {
                        Err(error) => eprintln!("Stream flushing error: {:?}", error),
                        Ok(_) => {
                            // Done
                        }
                    }
                }
            }
        }
    }
}

struct UnumDBCommandlineOptions {
    kv_engine: Option<KeyValueEngine>,
}

const DEFAULT_KV_ENGINE: KeyValueEngine = KeyValueEngine::HashMap;

fn parse_commandline_options(args: Vec<String>) -> UnumDBCommandlineOptions {
    let mut options = UnumDBCommandlineOptions { kv_engine: None };
    if args.len() > 2 {
        options.kv_engine = match args[1].as_ref() {
            "--redis" => Some(KeyValueEngine::Redis),
            "--memory" => Some(KeyValueEngine::HashMap),
            _ => None,
        }
    };
    options
}

fn main() {
    let commandline_options = parse_commandline_options(env::args().collect());
    let engine_choice = if let Some(choice) = commandline_options.kv_engine {
        choice
    } else {
        DEFAULT_KV_ENGINE
    };
    match TcpListener::bind(config::HTTP_ENDPOINT) {
        Err(error) => panic!(error),
        Ok(listener) => match UnumCore::new(engine_choice) {
            Err(_error) => eprintln!("Cannot create core"),
            Ok(mut core) => {
                for stream in listener.incoming() {
                    match stream {
                        Err(error) => eprintln!("Stream error {}", error),
                        Ok(stream) => handle_connection(stream, &mut core),
                    }
                }
            }
        },
    }
}
