use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use tiny_http::Server;

use crate::config::ImmuxDBConfiguration;
use crate::cortices::mongo::cortex::MONGO_CORTEX;
use crate::cortices::mysql::cortex::MYSQL_CORTEX;
use crate::cortices::unicus::cortex::responder;
use crate::cortices::{Cortex, CortexResponse};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::storage::core::ImmuxDBCore;

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
) -> ImmuxResult<()> {
    match stream.write(&data_to_client) {
        Err(error) => return Err(TcpError::TcpWriteError(error).into()),
        Ok(_bytes_written) => {
            let flushing = stream.flush();
            match flushing {
                Err(error) => {
                    return Err(TcpError::TcpFlushError(error).into());
                }
                Ok(_) => return Ok(()),
            }
        }
    }
}

fn handle_tcp_stream(
    mut stream: TcpStream,
    core: &mut ImmuxDBCore,
    bind_mode: &BindMode,
    config: &ImmuxDBConfiguration,
    cortex: &Cortex,
) -> ImmuxResult<()> {
    if let Some(process_first_connection_func) = cortex.process_first_connection {
        match process_first_connection_func(core) {
            Err(error) => return Err(error),
            Ok(success) => match success {
                CortexResponse::Send(data_to_client) => {
                    match send_data_to_stream_with_flushing(&stream, data_to_client) {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                }
                CortexResponse::SendThenDisconnect(data_to_client) => {
                    send_data_to_stream_with_flushing(&stream, data_to_client)?;
                    return Ok(());
                }
            },
        };
    }

    let mut buffer = [0; 1_024_000];
    loop {
        match stream.read(&mut buffer) {
            Err(error) => return Err(ImmuxError::Tcp(TcpError::TcpReadError(error))),
            Ok(bytes_read) => {
                println!("{} bytes read", bytes_read);
                if bytes_read > 0 {
                    match (cortex.process_incoming_message)(
                        &buffer[..bytes_read],
                        core,
                        &stream,
                        config,
                    ) {
                        Err(error) => return Err(error),
                        Ok(CortexResponse::Send(data_to_client)) => {
                            match send_data_to_stream_with_flushing(&stream, data_to_client) {
                                Err(error) => return Err(error),
                                Ok(_) => match bind_mode {
                                    BindMode::CloseAfterMessage => return Ok(()),
                                    BindMode::LongLive => continue,
                                },
                            };
                        }
                        Ok(CortexResponse::SendThenDisconnect(data_to_client)) => {
                            match send_data_to_stream_with_flushing(&stream, data_to_client) {
                                Err(error) => return Err(error),
                                Ok(_) => return Ok(()),
                            };
                        }
                    }
                }
            }
        }
    }
}

fn bind_tcp_port(
    endpoint: &str,
    core: &mut ImmuxDBCore,
    cortex: &Cortex,
    bind_mode: BindMode,
    config: &ImmuxDBConfiguration,
) -> ImmuxResult<()> {
    match TcpListener::bind(endpoint) {
        Err(error) => Err(ImmuxError::Tcp(TcpError::TcpBindError(error))),
        Ok(listener) => {
            for stream in listener.incoming() {
                match stream {
                    Err(error) => return Err(ImmuxError::Tcp(TcpError::TcpStreamError(error))),
                    Ok(stream) => match handle_tcp_stream(stream, core, &bind_mode, config, cortex)
                    {
                        Err(error) => return Err(error),
                        Ok(_) => {}
                    },
                };
            }
            return Ok(());
        }
    }
}

pub fn setup_cortices(mut core: ImmuxDBCore, config: &ImmuxDBConfiguration) -> ImmuxResult<()> {
    let server = Server::http("0.0.0.0:1991").unwrap();
    for mut request in server.incoming_requests() {
        responder(request, &mut core)?
    }

    bind_tcp_port(
        &config.mongo_endpoint,
        &mut core,
        &MONGO_CORTEX,
        BindMode::LongLive,
        config,
    )?;
    bind_tcp_port(
        &config.mysql_endpoint,
        &mut core,
        &MYSQL_CORTEX,
        BindMode::LongLive,
        config,
    )?;
    return Ok(());
}
