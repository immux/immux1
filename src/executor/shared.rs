use std::collections::HashMap;

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::declarations::errors::ImmuxResult;
use crate::declarations::instructions::{
    Answer, AtomicGetOneInstruction, AtomicSetInstruction, GetTargetSpec, Instruction,
    SetTargetSpec,
};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, ImmuxDBCore};

pub const SEPARATORS: &[u8] = &['/' as u8, '/' as u8];
pub const ID_LIST_KEY: &[u8] = &[
    'i' as u8, 'd' as u8, '_' as u8, 'l' as u8, 'i' as u8, 's' as u8, 't' as u8,
];
pub const INDEX_LIST_KEY: &[u8] = &[
    'i' as u8, 'n' as u8, 'd' as u8, 'e' as u8, 'x' as u8, '_' as u8, 'l' as u8, 'i' as u8,
    's' as u8, 't' as u8,
];
pub const STRING_IDENTIFIER: u8 = 0;
pub const BOOL_IDENTIFIER: u8 = 0;
pub const F64_IDENTIFIER: u8 = 0;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ValData {
    String(String),
    Bool(bool),
    Float64(f64),
}

pub type KeyList = Vec<Vec<u8>>;

pub fn construct_value_to_ids_map_from_js_obj(
    js_obj: &Value,
    index_field: &Vec<u8>,
    id: &Vec<u8>,
    grouping: &Vec<u8>,
    mut value_to_ids_map: &mut HashMap<Vec<u8>, Vec<Vec<u8>>>,
) -> ImmuxResult<()> {
    match &js_obj[&String::from_utf8_lossy(&index_field).into_owned()] {
        Value::String(string) => {
            let reverse_index_key = get_value_to_keys_map_key(
                grouping,
                &index_field,
                &ValData::String(string.clone()),
            )?;

            insert_to_vec_in_hashmap_with_default(
                &mut value_to_ids_map,
                reverse_index_key,
                id.clone(),
            );
        }
        Value::Bool(boolean) => {
            let reverse_index_key =
                get_value_to_keys_map_key(grouping, &index_field, &ValData::Bool(*boolean))?;
            insert_to_vec_in_hashmap_with_default(
                &mut value_to_ids_map,
                reverse_index_key,
                id.clone(),
            );
        }
        Value::Number(number) => {
            let mut number_f64 = 0.0;
            if let Some(num) = number.as_f64() {
                number_f64 = num;
            } else {
                return Err(ExecutorError::UnexpectedNumberType.into());
            }

            let reverse_index_key =
                get_value_to_keys_map_key(grouping, &index_field, &ValData::Float64(number_f64))?;
            insert_to_vec_in_hashmap_with_default(
                &mut value_to_ids_map,
                reverse_index_key,
                id.clone(),
            );
        }
        _ => {}
    }
    return Ok(());
}

pub fn insert_to_vec_in_hashmap_with_default(
    map: &mut HashMap<Vec<u8>, Vec<Vec<u8>>>,
    key: Vec<u8>,
    val: Vec<u8>,
) {
    match map.get(&key) {
        Some(value) => {
            let mut new_val = value.clone();
            new_val.push(val.clone());
            map.insert(key, new_val);
        }
        None => {
            let value = vec![val.clone()];
            map.insert(key, value);
        }
    }
}

pub fn get_value_to_keys_map_key(
    grouping: &[u8],
    key: &Vec<u8>,
    val_data: &ValData,
) -> ImmuxResult<Vec<u8>> {
    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(grouping);
    result.extend_from_slice(SEPARATORS);
    result.extend_from_slice(&key);
    result.extend_from_slice(SEPARATORS);
    match &val_data {
        ValData::String(string) => {
            result.push(STRING_IDENTIFIER);
            result.extend_from_slice(SEPARATORS);
            result.append(&mut string.as_bytes().to_vec());
        }
        ValData::Bool(boolean) => {
            result.push(BOOL_IDENTIFIER);
            result.extend_from_slice(SEPARATORS);
            match serialize(&boolean) {
                Err(_error) => return Err(ExecutorError::CannotSerialize.into()),
                Ok(mut data) => {
                    result.append(&mut data);
                }
            }
        }
        ValData::Float64(number_f64) => {
            result.push(F64_IDENTIFIER);
            result.extend_from_slice(SEPARATORS);
            match serialize(&number_f64) {
                Err(_error) => return Err(ExecutorError::CannotSerialize.into()),
                Ok(mut data) => {
                    result.append(&mut data);
                }
            }
        }
    }
    return Ok(result);
}

pub fn get_kv_key(collection: &[u8], key: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(collection);
    result.extend_from_slice(SEPARATORS);
    result.extend_from_slice(key);
    result
}

fn get_bytestring_vec(grouping: &[u8], list_key: Vec<u8>, core: &mut ImmuxDBCore) -> Vec<Vec<u8>> {
    let bytestring_vec = {
        let get_list = AtomicGetOneInstruction {
            target: GetTargetSpec {
                key: list_key.clone(),
                height: None,
            },
        };
        match core.execute(&Instruction::AtomicGetOne(get_list)) {
            Ok(Answer::GetOneOk(get_list_answer)) => {
                match deserialize::<KeyList>(&get_list_answer.item) {
                    Err(_error) => vec![],
                    Ok(key_list) => key_list,
                }
            }
            _ => vec![],
        }
    };
    bytestring_vec
}

fn save_internal_list(
    list_key: Vec<u8>,
    list: &[Vec<u8>],
    core: &mut ImmuxDBCore,
) -> ImmuxResult<()> {
    match serialize(&list) {
        Err(_error) => return Err(ExecutorError::CannotSerialize.into()),
        Ok(data) => {
            let update_list = AtomicSetInstruction {
                targets: vec![SetTargetSpec {
                    key: list_key.clone(),
                    value: data,
                }],
                increment_height: false,
            };
            match core.execute(&Instruction::AtomicSet(update_list)) {
                Err(error) => Err(error),
                Ok(_) => Ok(()),
            }
        }
    }
}

pub fn get_index_field_list(grouping: &[u8], core: &mut ImmuxDBCore) -> Vec<Vec<u8>> {
    let index_field_list_key = get_kv_key(grouping, INDEX_LIST_KEY);
    get_bytestring_vec(grouping, index_field_list_key, core)
}

pub fn set_index_field_list(
    grouping: &[u8],
    index_field_list: &[Vec<u8>],
    core: &mut ImmuxDBCore,
) -> ImmuxResult<()> {
    let index_filed_list_key = get_kv_key(grouping, INDEX_LIST_KEY);
    save_internal_list(index_filed_list_key, index_field_list, core)
}

pub fn get_id_list(grouping: &[u8], core: &mut ImmuxDBCore) -> Vec<Vec<u8>> {
    let id_list_key = get_kv_key(grouping, ID_LIST_KEY);
    get_bytestring_vec(grouping, id_list_key, core)
}

pub fn set_id_list(
    grouping: &[u8],
    core: &mut ImmuxDBCore,
    id_list: &[Vec<u8>],
) -> ImmuxResult<()> {
    let id_list_key = get_kv_key(grouping, ID_LIST_KEY);
    save_internal_list(id_list_key, id_list, core)
}

#[cfg(test)]
mod executor_shared_functions_test {
    use crate::config::DEFAULT_PERMANENCE_PATH;
    use crate::executor::shared::{get_id_list, get_kv_key, set_id_list};
    use crate::storage::core::ImmuxDBCore;
    use crate::storage::kv::KeyValueEngine;

    #[test]
    fn test_get_kv_key() {
        let collection = "collection";
        let id = "id";
        let kv_key = get_kv_key(collection.as_bytes(), id.as_bytes());
        assert_eq!(kv_key, "collection//id".as_bytes());
    }

    #[test]
    fn test_id_list() {
        let chain_name = "chain";
        match ImmuxDBCore::new(
            &KeyValueEngine::HashMap,
            DEFAULT_PERMANENCE_PATH,
            chain_name.as_bytes(),
        ) {
            Err(_error) => panic!("Cannot initialize core"),
            Ok(mut core) => {
                let collection = "collection".as_bytes();
                let input_list: Vec<Vec<u8>> = vec![
                    vec![0],
                    vec![1, 2, 3, 4, 5, 6],
                    "collection/yet-another-title".as_bytes().to_vec(),
                ];
                set_id_list(collection, &mut core, &input_list).unwrap();
                let output_list = get_id_list(collection, &mut core);
                assert_eq!(input_list, output_list);
            }
        }
    }
}
