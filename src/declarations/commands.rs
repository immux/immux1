use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertCommandSpec {
    pub id: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertCommand {
    pub grouping: Vec<u8>,
    pub targets: Vec<InsertCommandSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickChainCommand {
    pub new_chain_name: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SelectCondition {
    UnconditionalMatch,
    JSCode(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectCommand {
    pub grouping: Vec<u8>,
    pub condition: SelectCondition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    Insert(InsertCommand),
    PickChain(PickChainCommand),
    Select(SelectCommand),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertOutcome {
    pub count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickChainOutcome {
    pub new_chain_name: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectOutcome {
    pub values: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Outcome {
    Insert(InsertOutcome),
    PickChain(PickChainOutcome),
    Select(SelectOutcome),
}
