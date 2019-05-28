use std::net::TcpStream;

use crate::config::ImmuxDBConfiguration;
use crate::declarations::errors::ImmuxResult;
use crate::storage::core::ImmuxDBCore;

pub mod mongo;
pub mod mysql;
pub mod tcp;
pub mod unicus;
pub mod utils;

pub enum CortexResponse {
    Send(Vec<u8>),
    SendThenDisconnect(Vec<u8>),
}

pub struct Cortex {
    process_incoming_message: fn(
        bytes: &[u8],
        core: &mut ImmuxDBCore,
        stream: &TcpStream,
        config: &ImmuxDBConfiguration,
    ) -> ImmuxResult<CortexResponse>,
    process_first_connection: Option<fn(core: &mut ImmuxDBCore) -> ImmuxResult<CortexResponse>>,
}
