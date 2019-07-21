use crate::executor::shared::ValData;
use crate::storage::vkv::{Entry, InstructionHeight};
use serde::{Deserialize, Serialize};

/***************************************************
*
*                   Commands
*
***************************************************/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertCommandSpec {
    pub id: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertCommand {
    pub grouping: Vec<u8>,
    pub targets: Vec<InsertCommandSpec>,
    pub insert_with_index: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateIndexCommand {
    pub grouping: Vec<u8>,
    pub field: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickChainCommand {
    pub new_chain_name: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SelectCondition {
    UnconditionalMatch,
    Id(Vec<u8>),
    JSCode(String),
    Kv(Vec<u8>, ValData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectCommand {
    pub grouping: Vec<u8>,
    pub condition: SelectCondition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertCommandTargetSpec {
    pub id: Vec<u8>,
    pub target_height: InstructionHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertCommand {
    pub grouping: Vec<u8>,
    pub specs: Vec<RevertCommandTargetSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertAllCommand {
    pub target_height: InstructionHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InspectCommand {
    pub grouping: Vec<u8>,
    pub id: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    Insert(InsertCommand),
    PickChain(PickChainCommand),
    NameChain,
    Select(SelectCommand),
    CreateIndex(CreateIndexCommand),
    RevertOne(RevertCommand),
    RevertAll(RevertAllCommand),
    Inspect(InspectCommand),
}

/***************************************************
*
*                   Outcomes
*
***************************************************/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertOutcome {
    pub count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickChainOutcome {
    pub new_chain_name: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NameChainOutcome {
    pub chain_name: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectOutcome {
    pub values: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateIndexOutcome {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertOutcome {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertAllOutcome {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InspectOutcome {
    pub entry: Entry,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Outcome {
    Insert(InsertOutcome),
    PickChain(PickChainOutcome),
    Select(SelectOutcome),
    NameChain(NameChainOutcome),
    CreateIndex(CreateIndexOutcome),
    Revert(RevertOutcome),
    RevertAll(RevertAllOutcome),
    Inspect(InspectOutcome),
}
