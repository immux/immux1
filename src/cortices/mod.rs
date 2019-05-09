use std::net::TcpStream;

use crate::config::UnumDBConfiguration;
use crate::declarations::errors::UnumResult;
use crate::storage::core::UnumCore;

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
        core: &mut UnumCore,
        stream: &TcpStream,
        config: &UnumDBConfiguration,
    ) -> UnumResult<CortexResponse>,
    process_first_connection: Option<fn(core: &mut UnumCore) -> UnumResult<CortexResponse>>,
}
