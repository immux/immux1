use crate::storage::kv::KeyValueEngine;

pub const DB_VERSION: u8 = 1;

pub const UNICUS_ENDPOINT: &str = "127.0.0.1:1991";
pub const MONGO_ENDPOINT: &str = "127.0.0.1:27017";
pub const MYSQL_ENDPOINT: &str = "127.0.0.1:3306";

pub const HEIGHT_QUERY_KEYWORD: &str = "height";
pub const REVERT_QUERY_KEYWORD: &str = "revert";
pub const REVERTALL_QUERY_KEYWORD: &str = "revert_all";

const DEFAULT_KV_ENGINE: KeyValueEngine = KeyValueEngine::HashMap;

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

pub struct UnumDBConfiguration {
    pub engine_choice: KeyValueEngine,
    pub unicus_endpoint: &'static str,
    pub mongo_endpoint: &'static str,
    pub mysql_endpoint: &'static str,
}

pub fn compile_config(commandline_args: Vec<String>) -> UnumDBConfiguration {
    let commandline_options = parse_commandline_options(commandline_args);
    let engine_choice = if let Some(choice) = commandline_options.kv_engine {
        choice
    } else {
        DEFAULT_KV_ENGINE
    };
    UnumDBConfiguration {
        engine_choice,
        unicus_endpoint: UNICUS_ENDPOINT,
        mongo_endpoint: MONGO_ENDPOINT,
        mysql_endpoint: MYSQL_ENDPOINT,
    }
}
