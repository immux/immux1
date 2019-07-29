use bincode::deserialize;

use crate::declarations::commands::{Outcome, SelectCommand, SelectCondition, SelectOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{
    Answer, AtomicGetOneInstruction, GetTargetSpec, Instruction,
};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::{get_id_list, get_kv_key, get_value_to_keys_map_key};
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::kv::hashmap::HashmapStorageEngineError;
use crate::storage::kv::rocks::RocksEngineError;
use crate::utils;

pub fn execute_select(select: SelectCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    let mut values: Vec<Vec<u8>> = Vec::new();
    match &select.condition {
        SelectCondition::UnconditionalMatch => {
            let grouping = &select.grouping;
            let key_list = get_id_list(grouping, core);
            for key in key_list {
                println!("reading key {:#?}", utils::utf8_to_string(&key));
                let get_instruction = AtomicGetOneInstruction {
                    target: GetTargetSpec {
                        key: key.clone(),
                        height: None,
                    },
                };
                match core.execute(&Instruction::AtomicGetOne(get_instruction)) {
                    Err(error) => return Err(error),
                    Ok(Answer::GetOneOk(answer)) => {
                        let value = answer.item;
                        values.push(value);
                    }
                    Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
                }
            }
            Ok(Outcome::Select(SelectOutcome { values }))
        }
        SelectCondition::Id(id) => {
            let grouping = &select.grouping;
            let key = get_kv_key(grouping, id);
            let get_instruction = AtomicGetOneInstruction {
                target: GetTargetSpec { key, height: None },
            };
            match core.execute(&Instruction::AtomicGetOne(get_instruction)) {
                Err(error) => return Err(error),
                Ok(Answer::GetOneOk(answer)) => {
                    let value = answer.item;
                    values.push(value);
                    Ok(Outcome::Select(SelectOutcome { values }))
                }
                Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
            }
        }
        SelectCondition::Kv(key, val_data) => {
            let grouping = &select.grouping;
            let value_to_keys_map_key = get_value_to_keys_map_key(grouping, key, val_data)?;

            let get_instruction = AtomicGetOneInstruction {
                target: GetTargetSpec {
                    key: value_to_keys_map_key,
                    height: None,
                },
            };

            match core.execute(&Instruction::AtomicGetOne(get_instruction)) {
                Err(error) => match error {
                    ImmuxError::HashmapEngine(hashmap_storage_engine_error) => {
                        match hashmap_storage_engine_error {
                            HashmapStorageEngineError::NotFound => {}
                        }
                    }
                    ImmuxError::RocksEngine(rocks_engine_error) => match rocks_engine_error {
                        RocksEngineError::NotFound => {}
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
                            for key in id_list.iter() {
                                println!("reading key {:#?}", utils::utf8_to_string(&key));
                                let get_instruction = AtomicGetOneInstruction {
                                    target: GetTargetSpec {
                                        key: key.clone(),
                                        height: None,
                                    },
                                };

                                match core.execute(&Instruction::AtomicGetOne(get_instruction)) {
                                    Err(error) => {
                                        return Err(error);
                                    }
                                    Ok(Answer::GetOneOk(answer)) => {
                                        let value = answer.item;
                                        values.push(value);
                                    }
                                    Ok(answer) => {
                                        return Err(
                                            ExecutorError::UnexpectedAnswerType(answer).into()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
            }
            Ok(Outcome::Select(SelectOutcome { values }))
        }
        SelectCondition::JSCode(js_code) => {
            return Err(
                ExecutorError::UnimplementedSelectCondition(SelectCondition::JSCode(
                    js_code.to_owned(),
                ))
                .into(),
            );
        }
    }
}
