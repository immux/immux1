use std::convert::TryFrom;

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use crate::declarations::errors::{ImmuxError, ImmuxResult};

use crate::declarations::basics::db_version::DBVersion;
use crate::declarations::basics::{StoreKey, StoreValue};
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction, DataWriteInstruction,
    GetOneInstruction, Instruction, SetManyInstruction, SetTargetSpec,
};
use crate::storage::kv::KeyValueEngine;

pub const IMMUXDB_VERSION: u32 = 1;
pub static DB_VERSION: DBVersion = DBVersion::new(IMMUXDB_VERSION);

pub const UNICUS_ENDPOINT: &str = "127.0.0.1:1991";
pub const MONGO_ENDPOINT: &str = "127.0.0.1:27017";
pub const MYSQL_ENDPOINT: &str = "127.0.0.1:3306";

pub const INSPECT_KEYWORD: &str = "inspect";
pub const REVERT_QUERY_KEYWORD: &str = "revert";
pub const REVERTALL_QUERY_KEYWORD: &str = "revert_all";
pub const CHAIN_KEYWORD: &str = "chain";
pub const SELECT_CONDITION_KEYWORD: &str = "select";
pub const CREATE_INDEX_KEYWORD: &str = "index";
pub const INTERNAL_API_TARGET_ID_IDENTIFIER: &str = "internal_api_target_id_identifier";

pub const MULTIFIELD_SEPARATOR: &str = "|";

pub const DEFAULT_CHAIN_NAME: &str = "default";
pub const DEFAULT_PERMANENCE_PATH: &str = "/tmp/";

pub const INITIAL_TRANSACTION_ID_DATA: u64 = 1;

const DEFAULT_KV_ENGINE: KeyValueEngine = KeyValueEngine::Rocks;

pub const MAX_KVKEY_LENGTH: usize = 8 * 1024; // 8KB
pub const MAX_KVVALUE_LENGTH: usize = 32 * 1024 * 1024; // 32MB

pub const MAX_GROUPING_LABEL_LENGTH: usize = 128;

pub const MAX_CHAIN_NAME_LENGTH: usize = 128;

pub const MAX_PROPERTY_NAME_LENGTH: usize = 128;

const IS_MASTER: bool = true;
const MAX_MESSAGE_SIZE_BYTES: u32 = 48000000;
const MAX_WRITE_BATCH_SIZE: u32 = 100000;
const LOGICAL_SESSION_TIMEOUT_MINUTES: u32 = 30;
const MIN_MONGO_WIRE_VERSION: u32 = 0;
const MAX_MONGO_WIRE_VERSION: u32 = 7;
const READ_ONLY: bool = false;

pub const MAX_RECURSION: u16 = 128;

#[derive(Debug)]
pub enum ConfigError {
    CannotRead,
    UnexpectedCoreAnswer,
    CannotSerialize,
    CannotSet,
    CannotDeserialize,
    UnexpectedKeySigil(u8),
}

struct ImmuxDBCommandlineOptions {
    kv_engine: Option<KeyValueEngine>,
}

fn parse_commandline_options(args: &[String]) -> ImmuxDBCommandlineOptions {
    let mut options = ImmuxDBCommandlineOptions { kv_engine: None };
    if args.len() > 2 {
        options.kv_engine = match args[1].as_ref() {
            "--memory" => Some(KeyValueEngine::HashMap),
            _ => None,
        }
    };
    options
}

#[repr(u8)]
pub enum KVKeySigil {
    // Shared by whole chain
    ChainInfo = 0x10,
    ChainHeight = 0x11,

    // Shared by whole grouping
    GroupingInfo = 0x20,
    GroupingIndexedNames = 0x21,

    // By VKV
    UnitJournal = 0x30,
    HeightToInstructionRecord = 0x31,

    // By executor
    ReverseIndexIdList = 0xA0,
}

impl TryFrom<u8> for KVKeySigil {
    type Error = ConfigError;
    fn try_from(u: u8) -> Result<KVKeySigil, ConfigError> {
        if u == KVKeySigil::ChainInfo as u8 {
            return Ok(KVKeySigil::ChainInfo);
        } else if u == KVKeySigil::ChainHeight as u8 {
            return Ok(KVKeySigil::ChainHeight);
        } else if u == KVKeySigil::GroupingInfo as u8 {
            return Ok(KVKeySigil::GroupingInfo);
        } else if u == KVKeySigil::GroupingIndexedNames as u8 {
            return Ok(KVKeySigil::GroupingIndexedNames);
        } else if u == KVKeySigil::UnitJournal as u8 {
            return Ok(KVKeySigil::UnitJournal);
        } else if u == KVKeySigil::HeightToInstructionRecord as u8 {
            return Ok(KVKeySigil::HeightToInstructionRecord);
        } else if u == KVKeySigil::ReverseIndexIdList as u8 {
            return Ok(KVKeySigil::ReverseIndexIdList);
        } else {
            return Err(ConfigError::UnexpectedKeySigil(u));
        }
    }
}

impl From<KVKeySigil> for u8 {
    fn from(sigil: KVKeySigil) -> u8 {
        sigil as u8
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImmuxDBConfiguration {
    pub immuxdb_version: u32,
    pub engine_choice: KeyValueEngine,
    pub unicus_endpoint: String,
    pub mongo_endpoint: String,
    pub mysql_endpoint: String,
    pub data_root: String,

    pub is_master: bool,
    pub max_bson_object_size: u32,
    pub max_message_size_in_bytes: u32,
    pub max_write_batch_size: u32,
    pub logical_session_timeout_minutes: u32,
    pub min_mongo_wire_version: u32,
    pub max_mongo_wire_version: u32,
    pub read_only: bool,
}

impl Default for ImmuxDBConfiguration {
    fn default() -> Self {
        Self {
            immuxdb_version: IMMUXDB_VERSION,
            engine_choice: DEFAULT_KV_ENGINE,
            unicus_endpoint: UNICUS_ENDPOINT.to_string(),
            mongo_endpoint: MONGO_ENDPOINT.to_string(),
            mysql_endpoint: MYSQL_ENDPOINT.to_string(),
            data_root: DEFAULT_PERMANENCE_PATH.to_string(),
            is_master: IS_MASTER,
            max_bson_object_size: MAX_KVVALUE_LENGTH as u32,
            max_message_size_in_bytes: MAX_MESSAGE_SIZE_BYTES,
            max_write_batch_size: MAX_WRITE_BATCH_SIZE,
            logical_session_timeout_minutes: LOGICAL_SESSION_TIMEOUT_MINUTES,
            min_mongo_wire_version: MIN_MONGO_WIRE_VERSION,
            max_mongo_wire_version: MAX_MONGO_WIRE_VERSION,
            read_only: READ_ONLY,
        }
    }
}

impl ImmuxDBConfiguration {
    pub fn compile_from_args(commandline_args: &[String]) -> Self {
        let mut config = Self::default();
        let commandline_options = parse_commandline_options(commandline_args);
        if let Some(choice) = commandline_options.kv_engine {
            config.engine_choice = choice
        };
        config
    }
}

const GLOBAL_CONFIG_KEY: &str = "_CONFIG";

pub fn save_config(config: &ImmuxDBConfiguration, core: &mut ImmuxDBCore) -> ImmuxResult<()> {
    match serialize(&config) {
        Err(_error) => return Err(ImmuxError::Config(ConfigError::CannotSerialize)),
        Ok(bytes) => {
            let key: StoreKey = GLOBAL_CONFIG_KEY.as_bytes().into();
            let value = StoreValue::new(Some(bytes));
            let instruction = Instruction::DataAccess(DataInstruction::Write(
                DataWriteInstruction::SetMany(SetManyInstruction {
                    targets: vec![SetTargetSpec { key, value }],
                }),
            ));
            match core.execute(&instruction) {
                Err(_error) => Err(ImmuxError::Config(ConfigError::CannotSet)),
                Ok(_) => Ok(()),
            }
        }
    }
}

pub fn load_config(core: &mut ImmuxDBCore) -> ImmuxResult<ImmuxDBConfiguration> {
    let key: StoreKey = GLOBAL_CONFIG_KEY.as_bytes().into();
    let instruction = Instruction::DataAccess(DataInstruction::Read(DataReadInstruction::GetOne(
        GetOneInstruction { key, height: None },
    )));
    match core.execute(&instruction) {
        Err(_error) => return Err(ImmuxError::Config(ConfigError::CannotRead)),
        Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
            match answer.value.inner() {
                None => return Err(ConfigError::CannotRead.into()),
                Some(value) => match deserialize::<ImmuxDBConfiguration>(value) {
                    Err(_error) => return Err(ImmuxError::Config(ConfigError::CannotDeserialize)),
                    Ok(config) => return Ok(config),
                },
            }
        }
        _ => return Err(ImmuxError::Config(ConfigError::UnexpectedCoreAnswer)),
    }
}
