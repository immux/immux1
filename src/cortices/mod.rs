use crate::declarations::errors::UnumResult;
use crate::storage::core::UnumCore;

pub mod mongo;
pub mod mysql;
pub mod tcp;
pub mod unicus;

pub struct Cortex {
    process_incoming_message: fn(bytes: &[u8], core: &mut UnumCore) -> UnumResult<Option<Vec<u8>>>,
    process_first_connection: Option<fn(core: &mut UnumCore) -> UnumResult<Option<Vec<u8>>>>,
}
