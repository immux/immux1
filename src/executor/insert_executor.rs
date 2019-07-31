use crate::declarations::commands::{InsertCommand, InsertOutcome, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{
    Answer, AtomicGetOneInstruction, AtomicSetInstruction, GetTargetSpec, Instruction,
    SetTargetSpec,
};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::{
    construct_value_to_ids_map_from_js_obj, get_id_list, get_index_field_list, get_kv_key,
    set_id_list,
};
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::kv::hashmap::HashmapStorageEngineError;
use crate::storage::kv::rocks::RocksEngineError;

use bincode::{deserialize, serialize};
use serde_json::Value;
use std::collections::HashMap;

fn update_insert_targets(
    id_list: &Vec<Vec<u8>>,
    key: &Vec<u8>,
    targets: &mut Vec<SetTargetSpec>,
) -> ImmuxResult<()> {
    match serialize(id_list) {
        Err(_error) => return Err(ExecutorError::CannotSerialize.into()),
        Ok(data) => {
            let insert_command_spec = SetTargetSpec {
                key: key.clone(),
                value: data,
            };
            targets.push(insert_command_spec);
        }
    }
    return Ok(());
}

pub fn execute_insert(insert: InsertCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    let grouping = &insert.grouping;
    let mut key_list = get_id_list(grouping, core);
    let index_field_list = get_index_field_list(grouping, core);
    key_list.extend(
        insert
            .targets
            .iter()
            .map(|target| get_kv_key(grouping, &target.id)),
    );
    key_list.sort_by(|v1, v2| v1.cmp(v2));
    key_list.dedup_by(|v1, v2| v1 == v2);
    set_id_list(grouping, core, &key_list)?;
    let mut value_to_ids_map: HashMap<Vec<u8>, Vec<Vec<u8>>> = HashMap::new();

    let mut targets: Vec<SetTargetSpec> = insert
        .targets
        .iter()
        .map(|target| SetTargetSpec {
            key: get_kv_key(&grouping, &target.id),
            value: target.value.clone(),
        })
        .collect();

    if insert.insert_with_index {
        for target in &insert.targets {
            let json_string = String::from_utf8_lossy(&target.value.clone()).into_owned();
            let key = get_kv_key(grouping, &target.id);
            if let Ok(js_obj) = serde_json::from_str::<Value>(&json_string) {
                for index_field in index_field_list.iter() {
                    match construct_value_to_ids_map_from_js_obj(
                        &js_obj,
                        index_field,
                        &key,
                        grouping,
                        &mut value_to_ids_map,
                    ) {
                        Ok(()) => {}
                        Err(error) => {
                            return Err(error);
                        }
                    }
                }
            }
        }
    }

    for (key, val) in value_to_ids_map.iter() {
        let get_instruction = AtomicGetOneInstruction {
            target: GetTargetSpec {
                key: key.clone(),
                height: None,
            },
        };

        match core.execute(&Instruction::AtomicGetOne(get_instruction)) {
            Err(error) => match error {
                ImmuxError::HashmapEngine(hashmap_storage_engine_error) => {
                    match hashmap_storage_engine_error {
                        HashmapStorageEngineError::NotFound => {
                            update_insert_targets(val, key, &mut targets)?;
                        }
                    }
                }
                ImmuxError::RocksEngine(rocks_engine_error) => match rocks_engine_error {
                    RocksEngineError::NotFound => {
                        update_insert_targets(val, key, &mut targets)?;
                    }
                    _ => {
                        return Err(rocks_engine_error.into());
                    }
                },
                _ => {
                    return Err(error);
                }
            },
            Ok(Answer::GetOneOk(answer)) => {
                let value = answer.item;
                match deserialize::<Vec<Vec<u8>>>(&value) {
                    Err(_error) => {
                        return Err(ExecutorError::CannotDeserialize.into());
                    }
                    Ok(id_list) => {
                        let mut new_id_list = id_list.clone();
                        new_id_list.append(&mut val.clone());

                        update_insert_targets(&new_id_list, key, &mut targets)?;
                    }
                }
            }
            Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
        }
    }

    let store_data = AtomicSetInstruction {
        targets,
        increment_height: true,
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
