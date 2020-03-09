use std::convert::TryFrom;

use serde_json::Value as JsonValue;

use crate::declarations::basics::{
    GroupingLabel, IdList, PropertyNameList, StoreKey, StoreValue, Unit, UnitContent,
};
use crate::declarations::commands::{InsertCommand, InsertOutcome, Outcome};
use crate::declarations::errors::ImmuxResult;
use crate::executor::errors::ExecutorError;
use crate::executor::shared::{
    get_indexed_names_list_with_empty_fallback, get_store_key_of_indexed_id_list, ReverseIndex,
};
use crate::storage::core::CoreStore;
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction, DataWriteAnswer,
    DataWriteInstruction, GetOneInstruction, Instruction, SetManyInstruction, SetTargetSpec,
};

fn get_targets_existed_index(
    grouping: &GroupingLabel,
    property_names: &PropertyNameList,
    target_units: &[Unit],
    core: &mut impl CoreStore,
) -> ImmuxResult<ReverseIndex> {
    let mut reverse_index: ReverseIndex = ReverseIndex::new();
    for unit in target_units {
        let key = StoreKey::build(&grouping, unit.id.to_owned());
        let instruction = Instruction::DataAccess(DataInstruction::Read(
            DataReadInstruction::GetOne(GetOneInstruction { key, height: None }),
        ));
        match core.execute(&instruction) {
            Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                match answer.value.inner() {
                    Some(data) => {
                        let content = UnitContent::parse_data(data)?;
                        match content {
                            UnitContent::JsonString(json_string) => {
                                match serde_json::from_str::<JsonValue>(&json_string) {
                                    Err(_) => continue,
                                    Ok(json) => {
                                        for property_name in property_names.clone() {
                                            reverse_index.index_new_json(
                                                unit.id,
                                                &json,
                                                &property_name,
                                            )?;
                                        }
                                    }
                                }
                            }
                            _ => continue,
                        }
                    }
                    _ => continue,
                }
            }
            _ => continue,
        }
    }
    return Ok(reverse_index);
}

pub fn get_updates_for_index(
    grouping: &GroupingLabel,
    units: &[Unit],
    core: &mut impl CoreStore,
) -> ImmuxResult<Vec<SetTargetSpec>> {
    let indexed_names = get_indexed_names_list_with_empty_fallback(&grouping, core)?;

    let existed_index = get_targets_existed_index(&grouping, &indexed_names, &units, core)?;

    let new_index: ReverseIndex = {
        let mut index = ReverseIndex::new();
        for unit in units {
            match &unit.content {
                UnitContent::JsonString(json_string) => {
                    match serde_json::from_str::<JsonValue>(json_string) {
                        Err(_error) => continue,
                        Ok(json_value) => {
                            for name in indexed_names.clone() {
                                index.index_new_json(unit.id, &json_value, &name)?;
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        index
    };

    let mut updates_for_index = ReverseIndex::new();

    for ((name, property_bytes), ids_to_be_deleted) in existed_index {
        let property = UnitContent::parse_data(&property_bytes)?;
        let id_list_key = get_store_key_of_indexed_id_list(&grouping, &name, &property);
        let get_id_list = Instruction::DataAccess(DataInstruction::Read(
            DataReadInstruction::GetOne(GetOneInstruction {
                key: id_list_key.clone(),
                height: None,
            }),
        ));
        match core.execute(&get_id_list) {
            Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                match answer.value.inner() {
                    Some(data) => {
                        let mut existing_id_list = IdList::try_from(data.as_slice())?;
                        let (unit_content, _) = &UnitContent::parse(&property_bytes)?;
                        existing_id_list.remove_items(&ids_to_be_deleted);
                        updates_for_index.set(&name, unit_content, existing_id_list)?;
                    }
                    _ => continue,
                }
            }
            Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
            _ => continue,
        }
    }

    for ((name, property_bytes), new_ids) in new_index {
        let (unit_content, _) = &UnitContent::parse(&property_bytes)?;
        let property = UnitContent::parse_data(&property_bytes)?;
        if updates_for_index.get(&name, unit_content).is_empty() {
            let id_list_key = get_store_key_of_indexed_id_list(&grouping, &name, &property);
            let get_id_list = Instruction::DataAccess(DataInstruction::Read(
                DataReadInstruction::GetOne(GetOneInstruction {
                    key: id_list_key.clone(),
                    height: None,
                }),
            ));
            match core.execute(&get_id_list) {
                Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                    match answer.value.inner() {
                        None => return Err(ExecutorError::NoneReverseIndex.into()),
                        Some(data) => {
                            let mut existing_id_list = IdList::try_from(data.as_slice())?;
                            existing_id_list.merge(&new_ids);
                            updates_for_index.set(&name, unit_content, existing_id_list)?;
                        }
                    }
                }
                Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
                _ => continue,
            }
        } else {
            let mut ids = updates_for_index.get(&name, &property);
            ids.merge(&new_ids);
            updates_for_index.set(&name, unit_content, ids)?;
        }
    }

    let mut result: Vec<SetTargetSpec> = Vec::new();
    for ((name, property_bytes), new_ids) in updates_for_index {
        let property = UnitContent::parse_data(&property_bytes)?;
        let id_list_key = get_store_key_of_indexed_id_list(&grouping, &name, &property);
        result.push(SetTargetSpec {
            key: id_list_key,
            value: StoreValue::new(Some(new_ids.marshal())),
        })
    }

    return Ok(result);
}

pub fn execute_insert(insert: InsertCommand, core: &mut impl CoreStore) -> ImmuxResult<Outcome> {
    let original_insertions: Vec<SetTargetSpec> = insert
        .targets
        .iter()
        .map(|target| SetTargetSpec {
            key: StoreKey::build(&insert.grouping, target.id),
            value: StoreValue::new(Some(target.content.marshal())),
        })
        .collect();

    let units: Vec<Unit> = insert
        .targets
        .iter()
        .map(|insert_spec| {
            let id = insert_spec.id;
            let content = insert_spec.content.clone();
            return Unit { id, content };
        })
        .collect();
    let updates_for_index = get_updates_for_index(&insert.grouping, &units, core)?;

    let mut set_targets = Vec::new();
    set_targets.extend(original_insertions);
    set_targets.extend(updates_for_index);

    let batch_update: Instruction = Instruction::DataAccess(DataInstruction::Write(
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
