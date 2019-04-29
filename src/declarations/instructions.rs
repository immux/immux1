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
pub struct SwitchNamespaceInstruction {
    pub new_namespace: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadNamespaceInstruction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Instruction {
    Get(GetInstruction),
    Set(SetInstruction),
    Revert(RevertInstruction),
    RevertAll(RevertAllInstruction),
    SwitchNamespace(SwitchNamespaceInstruction),
    ReadNamespace(ReadNamespaceInstruction),
}

#[derive(Debug)]
pub struct GetOkAnswer {
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct SetOkAnswer {
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct RevertOkAnswer {
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct RevertAllOkAnswer {
    pub reverted_keys: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct SwitchNamespaceOkAnswer {
    pub new_namespace: Vec<u8>,
}

#[derive(Debug)]
pub struct ReadNamespaceOkAnswer {
    pub namespace: Vec<u8>,
}

#[derive(Debug)]
pub enum Answer {
    GetOk(GetOkAnswer),
    SetOk(SetOkAnswer),
    RevertOk(RevertOkAnswer),
    RevertAllOk(RevertAllOkAnswer),
    SwitchNamespaceOk(SwitchNamespaceOkAnswer),
    ReadNamespaceOk(ReadNamespaceOkAnswer),
}
