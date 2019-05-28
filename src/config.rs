use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use crate::declarations::errors::{UnumError, UnumResult};
use crate::declarations::instructions::Instruction::InTransactionSet;
use crate::declarations::instructions::{
    Answer, AtomicGetInstruction, AtomicGetOneInstruction, AtomicSetInstruction, GetTargetSpec,
    InTransactionSetInstruction, Instruction, SetTargetSpec,
};
use crate::storage::core::{CoreStore, UnumCore};
use crate::storage::kv::KeyValueEngine;

pub const UNUM_VERSION: u32 = 1;

pub const UNICUS_ENDPOINT: &str = "127.0.0.1:1991";
pub const MONGO_ENDPOINT: &str = "127.0.0.1:27017";
pub const MYSQL_ENDPOINT: &str = "127.0.0.1:3306";

pub const HEIGHT_QUERY_KEYWORD: &str = "height";
pub const REVERT_QUERY_KEYWORD: &str = "revert";
pub const REVERTALL_QUERY_KEYWORD: &str = "revert_all";
pub const CHAIN_KEYWORD: &str = "chain";

pub const DEFAULT_CHAIN_NAME: &str = "default";
pub const DEFAULT_PERMANENCE_PATH: &str = "/tmp/";

const DEFAULT_KV_ENGINE: KeyValueEngine = KeyValueEngine::HashMap;

const IS_MASTER: bool = true;
const MAX_BSON_OBJECT_SIZE: u32 = 16777216;
const MAX_MESSAGE_SIZE_BYTES: u32 = 48000000;
const MAX_WRITE_BATCH_SIZE: u32 = 100000;
const LOGICAL_SESSION_TIMEOUT_MINUTES: u32 = 30;
const MIN_MONGO_WIRE_VERSION: u32 = 0;
const MAX_MONGO_WIRE_VERSION: u32 = 7;
const READ_ONLY: bool = false;

#[derive(Debug)]
pub enum ConfigError {
    CannotRead,
    UnexpectedCoreAnswer,
    CannotSerialize,
    CannotSet,
    CannotDeserialize,
}

struct UnumDBCommandlineOptions {
    kv_engine: Option<KeyValueEngine>,
}

fn parse_commandline_options(args: Vec<String>) -> UnumDBCommandlineOptions {
    let mut options = UnumDBCommandlineOptions { kv_engine: None };
    if args.len() > 2 {
        options.kv_engine = match args[1].as_ref() {
            "--redis" => Some(KeyValueEngine::Redis),
            "--memory" => Some(KeyValueEngine::HashMap),
            _ => None,
        }
    };
    options
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnumDBConfiguration {
    pub unum_version: u32,
    pub engine_choice: KeyValueEngine,
    pub unicus_endpoint: String,
    pub mongo_endpoint: String,
    pub mysql_endpoint: String,

    pub is_master: bool,
    pub max_bson_object_size: u32,
    pub max_message_size_in_bytes: u32,
    pub max_write_batch_size: u32,
    pub logical_session_timeout_minutes: u32,
    pub min_mongo_wire_version: u32,
    pub max_mongo_wire_version: u32,
    pub read_only: bool,
}

pub fn compile_config(commandline_args: Vec<String>) -> UnumDBConfiguration {
    let mut config = UnumDBConfiguration {
        unum_version: UNUM_VERSION,
        engine_choice: DEFAULT_KV_ENGINE,
        unicus_endpoint: UNICUS_ENDPOINT.to_string(),
        mongo_endpoint: MONGO_ENDPOINT.to_string(),
        mysql_endpoint: MYSQL_ENDPOINT.to_string(),
        is_master: IS_MASTER,
        max_bson_object_size: MAX_BSON_OBJECT_SIZE,
        max_message_size_in_bytes: MAX_MESSAGE_SIZE_BYTES,
        max_write_batch_size: MAX_WRITE_BATCH_SIZE,
        logical_session_timeout_minutes: LOGICAL_SESSION_TIMEOUT_MINUTES,
        min_mongo_wire_version: MIN_MONGO_WIRE_VERSION,
        max_mongo_wire_version: MAX_MONGO_WIRE_VERSION,
        read_only: READ_ONLY,
    };
    let commandline_options = parse_commandline_options(commandline_args);
    if let Some(choice) = commandline_options.kv_engine {
        config.engine_choice = choice
    };
    config
}

const GLOBAL_CONFIG_KEY: &str = "_CONFIG";

pub fn save_config(config: &UnumDBConfiguration, core: &mut UnumCore) -> UnumResult<()> {
    match serialize(&config) {
        Err(_error) => return Err(UnumError::Config(ConfigError::CannotSerialize)),
        Ok(data) => {
            let instruction = AtomicSetInstruction {
                targets: vec![SetTargetSpec {
                    key: GLOBAL_CONFIG_KEY.as_bytes().to_vec(),
                    value: data,
                }],
            };
            match core.execute(&Instruction::AtomicSet(instruction)) {
                Err(_error) => Err(UnumError::Config(ConfigError::CannotSet)),
                Ok(_) => Ok(()),
            }
        }
    }
}

pub fn load_config(core: &mut UnumCore) -> UnumResult<UnumDBConfiguration> {
    let instruction = AtomicGetOneInstruction {
        target: GetTargetSpec {
            key: GLOBAL_CONFIG_KEY.as_bytes().to_vec(),
            height: None,
        },
    };
    match core.execute(&Instruction::AtomicGetOne(instruction)) {
        Err(_error) => return Err(UnumError::Config(ConfigError::CannotRead)),
        Ok(answer) => match answer {
            Answer::GetOneOk(get_answer) => {
                let target = &get_answer.item;
                match deserialize::<UnumDBConfiguration>(&target) {
                    Err(_error) => return Err(UnumError::Config(ConfigError::CannotDeserialize)),
                    Ok(config) => return Ok(config),
                }
            }
            _ => return Err(UnumError::Config(ConfigError::UnexpectedCoreAnswer)),
        },
    }
}
