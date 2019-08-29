use crate::declarations::commands::{Command, Outcome};
use crate::declarations::errors::ImmuxResult;

use crate::executor::create_index_executor::execute_create_index;
use crate::executor::insert_executor::execute_insert;
use crate::executor::inspect_executor::execute_inspect;
use crate::executor::name_chain_executor::execute_name_chain;
use crate::executor::pick_chain_executor::execute_pick_chain;
use crate::executor::revert_all_executor::execute_revert_all;
use crate::executor::revert_executor::execute_revert_one;
use crate::executor::select_executor::execute_select;
use crate::storage::core::ImmuxDBCore;

pub fn execute(command: Command, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    match command {
        Command::PickChain(pick_chain) => execute_pick_chain(pick_chain, core),
        Command::Insert(insert) => execute_insert(insert, core),
        Command::Select(select) => execute_select(select, core),
        Command::NameChain => execute_name_chain(core),
        Command::CreateIndex(create_index) => execute_create_index(create_index, core),
        Command::RevertOne(revert) => execute_revert_one(revert, core),
        Command::RevertAll(revert_all) => execute_revert_all(revert_all, core),
        Command::Inspect(inspect) => execute_inspect(inspect, core),
    }
}
