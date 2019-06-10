use crate::declarations::commands::{InsertCommand, InsertOutcome, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{Answer, AtomicSetInstruction, Instruction, SetTargetSpec};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::{get_id_list, get_kv_key, set_id_list};
use crate::storage::core::{CoreStore, ImmuxDBCore};

pub fn execute_insert(insert: InsertCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    let grouping = &insert.grouping;
    let mut key_list = get_id_list(grouping, core);
    key_list.extend(
        insert
            .targets
            .iter()
            .map(|target| get_kv_key(grouping, &target.id)),
    );
    key_list.sort_by(|v1, v2| v1.cmp(v2));
    key_list.dedup_by(|v1, v2| v1 == v2);
    set_id_list(grouping, core, &key_list)?;

    let store_data = AtomicSetInstruction {
        targets: insert
            .targets
            .iter()
            .map(|target| SetTargetSpec {
                key: get_kv_key(&grouping, &target.id),
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
                return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                    answer,
                )));
            }
        },
    }
}
