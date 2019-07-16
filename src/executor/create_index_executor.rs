use crate::declarations::commands::{CreateIndexCommand, CreateIndexOutcome, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{
    Answer, AtomicGetOneInstruction, AtomicSetInstruction, GetTargetSpec, Instruction,
    SetTargetSpec,
};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::{
    construct_value_to_ids_map_from_js_obj, get_id_list, get_index_field_list,
    set_index_field_list,
};
use crate::storage::core::{CoreStore, ImmuxDBCore};

use bincode::serialize;
use serde_json::Value;
use std::collections::HashMap;

pub fn execute_create_index(
    create_index: CreateIndexCommand,
    core: &mut ImmuxDBCore,
) -> ImmuxResult<Outcome> {
    let grouping = &create_index.grouping;
    let index_field = String::from_utf8_lossy(&create_index.field).into_owned();

    let mut index_field_list = get_index_field_list(grouping, core);
    index_field_list.push(index_field.clone().as_bytes().to_vec());
    index_field_list.sort_by(|v1, v2| v1.cmp(v2));
    index_field_list.dedup_by(|v1, v2| v1 == v2);
    set_index_field_list(grouping, &index_field_list, core)?;

    let mut value_to_ids_map: HashMap<Vec<u8>, Vec<Vec<u8>>> = HashMap::new();

    let key_list = get_id_list(grouping, core);
    for key in key_list.iter() {
        let get_instruction = AtomicGetOneInstruction {
            target: GetTargetSpec {
                key: key.clone(),
                height: None,
            },
        };

        match core.execute(&Instruction::AtomicGetOne(get_instruction)) {
            Err(error) => return Err(error),
            Ok(Answer::GetOneOk(answer)) => {
                let json_string = String::from_utf8_lossy(&answer.item).into_owned();
                match serde_json::from_str::<Value>(&json_string) {
                    Ok(js_obj) => {
                        match construct_value_to_ids_map_from_js_obj(
                            &js_obj,
                            &create_index.field,
                            key,
                            grouping,
                            &mut value_to_ids_map,
                        ) {
                            Ok(()) => {}
                            Err(error) => {
                                return Err(error);
                            }
                        }
                    }
                    Err(_error) => {
                        continue;
                    }
                }
            }
            Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
        }
    }

    let mut targets = Vec::new();
    for (index_key, index_val) in value_to_ids_map.iter() {
        match serialize(&index_val) {
            Err(_error) => return Err(ExecutorError::CannotSerialize.into()),
            Ok(data) => {
                let insert_command_spec = SetTargetSpec {
                    key: index_key.to_vec(),
                    value: data,
                };
                targets.push(insert_command_spec);
            }
        }
    }

    let store_data = AtomicSetInstruction {
        targets,
        increment_height: false,
    };

    match core.execute(&Instruction::AtomicSet(store_data)) {
        Err(error) => return Err(error),
        Ok(answer) => match answer {
            Answer::SetOk(_answer) => {
                return Ok(Outcome::CreateIndex(CreateIndexOutcome {}));
            }
            _ => {
                return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                    answer,
                )));
            }
        },
    }
}
