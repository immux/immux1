pub mod config;
pub mod cortices;
pub mod declarations;
pub mod executor;
pub mod storage;
pub mod utils;

use std::env;

use crate::config::{compile_config, save_config, ImmuxDBConfiguration, DEFAULT_CHAIN_NAME};
use crate::cortices::tcp::setup_cortices;
use crate::declarations::errors::ImmuxResult;
use crate::storage::core::ImmuxDBCore;

fn initialize(config: ImmuxDBConfiguration) -> ImmuxResult<()> {
    let config = compile_config(env::args().collect());
    let mut core = ImmuxDBCore::new(&config.engine_choice, DEFAULT_CHAIN_NAME.as_bytes())?;
    save_config(&config, &mut core)?;
    setup_cortices(core, &config)?;
    return Ok(());
}

pub fn run_immuxdb() {
    let config = compile_config(env::args().collect());
    match initialize(config) {
        Err(error) => eprintln!("ImmuxDB failed: {:#?}", error),
        Ok(_) => (),
    }
}
