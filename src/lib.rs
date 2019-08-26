pub mod config;
pub mod cortices;
pub mod declarations;
pub mod executor;
pub mod storage;
pub mod utils;

use crate::config::{save_config, ImmuxDBConfiguration, DEFAULT_CHAIN_NAME};
use crate::cortices::tcp::setup_cortices;
use crate::declarations::errors::ImmuxResult;
use crate::storage::core::ImmuxDBCore;
use crate::storage::instructions::StoreNamespace;

pub fn run_immuxdb(config: &ImmuxDBConfiguration) -> ImmuxResult<()> {
    let mut core = ImmuxDBCore::new(
        &config.engine_choice,
        &config.data_root,
        &StoreNamespace::new(DEFAULT_CHAIN_NAME.as_bytes()),
    )?;
    save_config(config, &mut core)?;
    setup_cortices(core, config)?;
    return Ok(());
}

pub fn run_immuxdb_with_default_config() -> ImmuxResult<()> {
    let config = ImmuxDBConfiguration::default();
    run_immuxdb(&config)
}
