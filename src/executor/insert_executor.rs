use std::convert::TryFrom;

use serde_json::Value as JsonValue;

use crate::declarations::basics::{IdList, StoreKey, StoreValue, UnitContent};
use crate::declarations::commands::{InsertCommand, InsertOutcome, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::executor::reverse_index::ReverseIndex;
use crate::executor::shared::{
    get_indexed_names_list_with_fallback, get_store_key_of_indexed_id_list,
};
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction, DataWriteAnswer,
    DataWriteInstruction, GetOneInstruction, Instruction, SetManyInstruction, SetTargetSpec,
};
use crate::storage::kv::KVError;

pub fn get_updates_for_index(
    insert: InsertCommand,
    core: &mut ImmuxDBCore,
) -> ImmuxResult<Vec<SetTargetSpec>> {
    let indexed_names = get_indexed_names_list_with_fallback(&insert.grouping, core)?;
    let reverse_index: ReverseIndex = {
        let mut index = ReverseIndex::new();
        for target in &insert.targets {
            match &target.content {
                UnitContent::JsonString(json_string) => {
                    match serde_json::from_str::<JsonValue>(json_string) {
                        Err(_error) => continue,
                        Ok(json_value) => {
                            for name in indexed_names.clone() {
                                index.index_new_json(target.id, &json_value, &name)?;
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        index
    };

    let mut updates_for_index: Vec<SetTargetSpec> = Vec::new();

    for ((name, property_bytes), new_ids) in reverse_index {
        let property = UnitContent::parse(&property_bytes)?;
        let id_list_key = get_store_key_of_indexed_id_list(&insert.grouping, &name, &property);
        let get_id_list = Instruction::Data(DataInstruction::Read(DataReadInstruction::GetOne(
            GetOneInstruction {
                key: id_list_key.clone(),
                height: None,
            },
        )));
        match core.execute(&get_id_list) {
            Err(ImmuxError::KV(KVError::NotFound(_))) => updates_for_index.push(SetTargetSpec {
                key: id_list_key,
                value: StoreValue::new(new_ids.marshal()),
            }),
            Err(error) => {
                return Err(error.into());
            }
            Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                let existing_id_list_bytes: Vec<u8> = answer.value.into();
                let mut existing_id_list = IdList::try_from(existing_id_list_bytes.as_slice())?;
                existing_id_list.merge(&new_ids);
                updates_for_index.push(SetTargetSpec {
                    key: id_list_key,
                    value: StoreValue::new(existing_id_list.marshal()),
                });
            }
            Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
        }
    }

    return Ok(updates_for_index);
}

pub fn execute_insert(insert: InsertCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    let original_insertions: Vec<SetTargetSpec> = insert
        .targets
        .iter()
        .map(|target| SetTargetSpec {
            key: StoreKey::build(&insert.grouping, target.id),
            value: target.content.marshal().into(),
        })
        .collect();

    let updates_for_index = get_updates_for_index(insert, core)?;

    let mut set_targets = Vec::new();
    set_targets.extend(original_insertions);
    set_targets.extend(updates_for_index);

    let batch_update: Instruction = Instruction::Data(DataInstruction::Write(
        DataWriteInstruction::SetMany(SetManyInstruction {
            targets: set_targets,
        }),
    ));

    match core.execute(&batch_update) {
        Err(error) => return Err(error),
        Ok(Answer::DataAccess(DataAnswer::Write(DataWriteAnswer::SetOk(answer)))) => {
            return Ok(Outcome::Insert(InsertOutcome {
                count: answer.count,
            }));
        }
        Ok(answer) => {
            return Err(ExecutorError::UnexpectedAnswerType(answer).into());
        }
    }
}
