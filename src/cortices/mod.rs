use crate::cortices::mysql::cortex::{
    mysql_cortex_process_first_connection, mysql_cortex_process_incoming_message,
};

use crate::cortices::mongo::cortex::mongo_cortex_process_incoming_message;
use crate::cortices::unicus::cortex::unicus_cortex_process_incoming_message;
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

pub fn get_mysql_cortex() -> Cortex {
    Cortex {
        process_incoming_message: mysql_cortex_process_incoming_message,
        process_first_connection: Some(mysql_cortex_process_first_connection),
    }
}

pub fn get_mongo_cortex() -> Cortex {
    Cortex {
        process_incoming_message: mongo_cortex_process_incoming_message,
        process_first_connection: None,
    }
}

pub fn get_unicus_cortex() -> Cortex {
    Cortex {
        process_incoming_message: unicus_cortex_process_incoming_message,
        process_first_connection: None,
    }
}
