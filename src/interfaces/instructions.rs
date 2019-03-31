use serde::{Deserialize, Serialize};

use crate::storage::vkv::InstructionHeight;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTargetSpec {
    pub key: Vec<u8>,
    pub height: Option<InstructionHeight>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetTargetSpec {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertTargetSpec {
    pub key: Vec<u8>,
    pub height: InstructionHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetInstruction {
    pub targets: Vec<GetTargetSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetInstruction {
    pub targets: Vec<SetTargetSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertInstruction {
    pub targets: Vec<RevertTargetSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertAllInstruction {
    pub target_height: InstructionHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Instruction {
    Get(GetInstruction),
    Set(SetInstruction),
    Revert(RevertInstruction),
    RevertAll(RevertAllInstruction),
}

pub struct GetOkAnswer {
    pub items: Vec<Vec<u8>>,
}

pub struct SetOkAnswer {
    pub items: Vec<Vec<u8>>,
}

pub struct RevertOkAnswer {
    pub items: Vec<Vec<u8>>,
}

pub struct RevertAllOkAnswer {
    pub reverted_keys: Vec<Vec<u8>>,
}

pub enum Answer {
    GetOk(GetOkAnswer),
    SetOk(SetOkAnswer),
    RevertOk(RevertOkAnswer),
    RevertAllOk(RevertAllOkAnswer),
}
