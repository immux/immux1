use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::config::UnumDBConfiguration;
use crate::cortices::mongo::cortex::MONGO_CORTEX;
use crate::cortices::mysql::cortex::MYSQL_CORTEX;
use crate::cortices::unicus::cortex::UNICUS_CORTEX;
use crate::cortices::Cortex;
use crate::declarations::errors::{UnumError, UnumResult};
use crate::storage::core::UnumCore;

#[derive(Debug)]
pub enum TcpError {
    TcpBindError(std::io::Error),
    TcpStreamError(std::io::Error),
    TcpReadError(std::io::Error),
    TcpWriteError(std::io::Error),
    TcpFlushError(std::io::Error),
}

pub enum BindMode {
    LongLive,          // Generic TCP
    CloseAfterMessage, // HTTP-like, close after each message
}

fn send_data_to_stream_with_flushing(
    mut stream: &TcpStream,
    data_to_client: Vec<u8>,
) -> UnumResult<()> {
    match stream.write(&data_to_client) {
        Err(error) => {
            return Err(UnumError::Tcp(TcpError::TcpWriteError(error)));
        }
        Ok(_bytes_written) => {
            let flushing = stream.flush();
            match flushing {
                Err(error) => {
                    return Err(UnumError::Tcp(TcpError::TcpFlushError(error)));
                }
                Ok(_) => return Ok(()),
            }
        }
    }
}

fn handle_tcp_stream(
    mut stream: TcpStream,
    core: &mut UnumCore,
    bind_mode: &BindMode,
    cortex: &Cortex,
) -> UnumResult<()> {
    if let Some(process_first_connection_func) = cortex.process_first_connection {
        match process_first_connection_func(core) {
            Err(error) => return Err(error),
            Ok(success) => match success {
                None => return Ok(()),
                Some(data_to_client) => {
                    match send_data_to_stream_with_flushing(&stream, data_to_client) {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                }
            },
        };
    }

    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Err(error) => return Err(UnumError::Tcp(TcpError::TcpReadError(error))),
            Ok(bytes_read) => {
                match (cortex.process_incoming_message)(&buffer[..bytes_read], core) {
                    Err(error) => return Err(error),
                    Ok(success) => match success {
                        None => return Ok(()),
                        Some(data_to_client) => {
                            match send_data_to_stream_with_flushing(&stream, data_to_client) {
                                Err(error) => return Err(error),
                                Ok(_) => match bind_mode {
                                    BindMode::CloseAfterMessage => return Ok(()),
                                    BindMode::LongLive => continue,
                                },
                            }
                        }
                    },
                }
            }
        }
    }
}

fn bind_tcp_port(
    endpoint: &str,
    core: &mut UnumCore,
    processor: &Cortex,
    bind_mode: BindMode,
) -> UnumResult<()> {
    match TcpListener::bind(endpoint) {
        Err(error) => Err(UnumError::Tcp(TcpError::TcpBindError(error))),
        Ok(listener) => {
            for stream in listener.incoming() {
                match stream {
                    Err(error) => return Err(UnumError::Tcp(TcpError::TcpStreamError(error))),
                    Ok(stream) => match handle_tcp_stream(stream, core, &bind_mode, processor) {
                        Err(error) => eprintln!("TCP error: {:#?}", error),
                        Ok(_) => {}
                    },
                };
            }
            return Ok(());
        }
    }
}

pub fn setup_cortices(mut core: UnumCore, config: &UnumDBConfiguration) -> UnumResult<()> {
    //     TODO(#30): bind_tcp_port() blocks; only the first takes effect
    //    bind_tcp_port(
    //        &config.mysql_endpoint,
    //        &mut core,
    //        &MYSQL_CORTEX,
    //        BindMode::LongLive,
    //    )?;
    bind_tcp_port(
        &config.mongo_endpoint,
        &mut core,
        &MONGO_CORTEX,
        BindMode::LongLive,
    )?;
    bind_tcp_port(
        &config.unicus_endpoint,
        &mut core,
        &UNICUS_CORTEX,
        BindMode::CloseAfterMessage,
    )?;
    return Ok(());
}
