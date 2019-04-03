mod config;
mod cortices;
mod declarations;
mod storage;
mod utils;

use std::env;

use crate::config::compile_config;
use crate::cortices::tcp::setup_cortices;
use crate::declarations::errors::UnumResult;
use crate::storage::core::UnumCore;

fn initialize() -> UnumResult<()> {
    let config = compile_config(env::args().collect());
    let core = UnumCore::new(&config.engine_choice)?;
    setup_cortices(core, &config)?;
    return Ok(());
}

fn main() {
    match initialize() {
        Err(error) => eprintln!("UnumDB: initialization failed: {:#?}", error),
        Ok(_) => (),
    }
}
