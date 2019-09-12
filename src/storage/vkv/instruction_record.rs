use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::DB_VERSION;
use crate::declarations::basics::db_version::DBVersion;
use crate::declarations::basics::StoreKey;
use crate::storage::instructions::Instruction;

fn now() -> u128 {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros())
        .unwrap_or(0)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstructionRecord {
    pub instruction: Instruction,
    pub version: DBVersion,
    pub sys_time: u128,

    // Only for some instructions that do not include all keys that would be affected
    pub affected_keys: Option<Vec<StoreKey>>,
}

impl From<Instruction> for InstructionRecord {
    fn from(instruction: Instruction) -> InstructionRecord {
        InstructionRecord {
            instruction,
            version: DB_VERSION,
            sys_time: now(),
            affected_keys: None,
        }
    }
}
