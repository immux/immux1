use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::config::UnumDBConfiguration;
use crate::cortices::mongo::cortex::mongo_cortex;
use crate::cortices::unicus::cortex::unicus_cortex;
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

pub type TcpMessageProcessFn = Fn(&[u8], &mut UnumCore) -> UnumResult<Option<Vec<u8>>>;

fn handle_tcp_stream(
    mut stream: TcpStream,
    core: &mut UnumCore,
    process_message: &TcpMessageProcessFn,
) -> UnumResult<()> {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Err(error) => Err(UnumError::Tcp(TcpError::TcpReadError(error))),
        Ok(bytes_read) => match process_message(&buffer[..bytes_read], core) {
            Err(error) => return Err(error),
            Ok(success) => match success {
                None => return Ok(()),
                Some(data_to_client) => match stream.write(&data_to_client) {
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
                },
            },
        },
    }
}

fn bind_tcp_port(
    endpoint: &str,
    core: &mut UnumCore,
    processor: &TcpMessageProcessFn,
) -> UnumResult<()> {
    match TcpListener::bind(endpoint) {
        Err(error) => Err(UnumError::Tcp(TcpError::TcpBindError(error))),
        Ok(listener) => {
            for stream in listener.incoming() {
                match stream {
                    Err(error) => return Err(UnumError::Tcp(TcpError::TcpStreamError(error))),
                    Ok(stream) => match handle_tcp_stream(stream, core, processor) {
                        Err(error) => eprintln!("TCP error: {:#?}", error),
                        Ok(_) => (),
                    },
                };
            }
            return Ok(());
        }
    }
}

pub fn setup_cortices(mut core: UnumCore, config: &UnumDBConfiguration) -> UnumResult<()> {
    // TODO(#30): bind_tcp_port() blocks; only the first takes effect
    bind_tcp_port(config.mongo_endpoint, &mut core, &mongo_cortex)?;
    bind_tcp_port(config.unicus_endpoint, &mut core, &unicus_cortex)?;
    return Ok(());
}
