mod config;
mod cortices;
mod declarations;
mod executor;
mod storage;
mod utils;

use std::env;

use crate::config::{compile_config, save_config, DEFAULT_CHAIN_NAME};
use crate::cortices::tcp::setup_cortices;
use crate::declarations::errors::UnumResult;
use crate::storage::core::UnumCore;

fn initialize() -> UnumResult<()> {
    let config = compile_config(env::args().collect());
    let mut core = UnumCore::new(&config.engine_choice, DEFAULT_CHAIN_NAME.as_bytes())?;
    save_config(&config, &mut core)?;
    setup_cortices(core, &config)?;
    return Ok(());
}

fn main() {
    match initialize() {
        Err(error) => eprintln!("UnumDB failed: {:#?}", error),
        Ok(_) => (),
    }
}
