use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::config::UnumDBConfiguration;
use crate::cortices::{get_mongo_cortex, get_mysql_cortex, get_unicus_cortex, Cortex};
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

fn process_incoming_bytes(
    mut stream: &TcpStream,
    core: &mut UnumCore,
    cortex: &Cortex,
) -> UnumResult<()> {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Err(error) => Err(UnumError::Tcp(TcpError::TcpReadError(error))),
        Ok(bytes_read) => match (cortex.process_incoming_message)(&buffer[..bytes_read], core) {
            Err(error) => return Err(error),
            Ok(success) => match success {
                None => return Ok(()),
                Some(data_to_client) => send_data_to_stream_with_flushing(&stream, data_to_client),
            },
        },
    }
}

fn handle_tcp_stream(
    stream: TcpStream,
    core: &mut UnumCore,
    cortex: &Cortex,
    is_immediately_after_connection: bool,
) -> UnumResult<()> {
    if is_immediately_after_connection {
        if let Some(process_first_connection_func) = cortex.process_first_connection {
            match process_first_connection_func(core) {
                Err(error) => return Err(error),
                Ok(success) => match success {
                    None => return Ok(()),
                    Some(data_to_client) => {
                        send_data_to_stream_with_flushing(&stream, data_to_client)
                    }
                },
            }
        } else {
            process_incoming_bytes(&stream, core, &cortex)
        }
    } else {
        process_incoming_bytes(&stream, core, &cortex)
    }
}

fn bind_tcp_port(endpoint: &str, core: &mut UnumCore, processor: &Cortex) -> UnumResult<()> {
    match TcpListener::bind(endpoint) {
        Err(error) => Err(UnumError::Tcp(TcpError::TcpBindError(error))),
        Ok(listener) => {
            let mut is_immediately_after_connection = true;
            for stream in listener.incoming() {
                match stream {
                    Err(error) => return Err(UnumError::Tcp(TcpError::TcpStreamError(error))),
                    Ok(stream) => {
                        match handle_tcp_stream(
                            stream,
                            core,
                            processor,
                            is_immediately_after_connection,
                        ) {
                            Err(error) => eprintln!("TCP error: {:#?}", error),
                            Ok(_) => {
                                is_immediately_after_connection = false;
                            }
                        }
                    }
                };
            }
            return Ok(());
        }
    }
}

pub fn setup_cortices(mut core: UnumCore, config: &UnumDBConfiguration) -> UnumResult<()> {
    //     TODO(#30): bind_tcp_port() blocks; only the first takes effect
    bind_tcp_port(&config.mysql_endpoint, &mut core, &get_mysql_cortex())?;
    bind_tcp_port(&config.mongo_endpoint, &mut core, &get_mongo_cortex())?;
    bind_tcp_port(&config.unicus_endpoint, &mut core, &get_unicus_cortex())?;
    return Ok(());
}
