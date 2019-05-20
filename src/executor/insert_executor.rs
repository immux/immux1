use crate::declarations::commands::{InsertCommand, InsertOutcome, Outcome};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::declarations::instructions::{Answer, AtomicSetInstruction, Instruction, SetTargetSpec};
use crate::executor::execute::ExecutorError;
use crate::executor::shared::{get_id_list, get_kv_key, set_id_list};
use crate::storage::core::{CoreStore, UnumCore};

pub fn execute_insert(insert: InsertCommand, core: &mut UnumCore) -> UnumResult<Outcome> {
    // TODO(#79): Currently, insert targets are assumed to go to one collection
    let grouping = &insert.targets[0].grouping;
    let mut key_list = get_id_list(grouping, core);
    key_list.extend(
        insert
            .targets
            .iter()
            .map(|target| get_kv_key(grouping, &target.id)),
    );
    set_id_list(grouping, core, &key_list)?;

    let store_data = AtomicSetInstruction {
        targets: insert
            .targets
            .iter()
            .map(|target| SetTargetSpec {
                key: get_kv_key(&target.grouping, &target.id),
                value: target.value.clone(),
            })
            .collect(),
    };
    match core.execute(&Instruction::AtomicSet(store_data)) {
        Err(error) => return Err(error),
        Ok(answer) => match answer {
            Answer::SetOk(answer) => {
                return Ok(Outcome::Insert(InsertOutcome {
                    count: answer.items.len(),
                }));
            }
            _ => {
                return Err(UnumError::Executor(ExecutorError::UnexpectedAnswerType(
                    answer,
                )));
            }
        },
    }
}
