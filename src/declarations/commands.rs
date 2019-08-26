use serde::{Deserialize, Serialize};

use crate::declarations::basics::{
    ChainName, GroupingLabel, PropertyName, Unit, UnitContent, UnitId, UnitSpecifier,
};
use crate::storage::vkv::ChainHeight;

/***************************************************
*
*                   Commands
*
***************************************************/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertCommandSpec {
    pub id: UnitId,
    pub content: UnitContent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertCommand {
    pub grouping: GroupingLabel,
    pub targets: Vec<InsertCommandSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateIndexCommand {
    pub grouping: GroupingLabel,
    pub name: PropertyName,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickChainCommand {
    pub new_chain_name: ChainName,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SelectCondition {
    UnconditionalMatch,
    Id(UnitId),
    JSCode(String),
    NameProperty(PropertyName, UnitContent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectCommand {
    pub grouping: GroupingLabel,
    pub condition: SelectCondition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertCommandTargetSpec {
    pub specifier: UnitSpecifier,
    pub target_height: ChainHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertCommand {
    pub specs: Vec<RevertCommandTargetSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertAllCommand {
    pub target_height: ChainHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InspectCommand {
    pub specifier: UnitSpecifier,
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
    pub new_chain_name: ChainName,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NameChainOutcome {
    pub chain_name: ChainName,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectOutcome {
    pub units: Vec<Unit>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateIndexOutcome {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertOutcome {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertAllOutcome {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Inspection {
    pub deleted: bool,
    pub height: ChainHeight,
    pub current_content: UnitContent,
}

impl ToString for Inspection {
    fn to_string(&self) -> String {
        format!(
            "{}|{}|{}",
            self.deleted,
            self.height.as_u64(),
            self.current_content.to_string()
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InspectOutcome {
    pub inspections: Vec<Inspection>,
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
