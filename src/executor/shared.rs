use bincode::{deserialize, serialize};

use crate::declarations::errors::{UnumError, UnumResult};
use crate::declarations::instructions::{
    Answer, AtomicGetInstruction, AtomicSetInstruction, GetTargetSpec, Instruction, SetTargetSpec,
};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, UnumCore};

pub const SEPARATORS: &[u8] = &['/' as u8, '/' as u8];
pub const ID_LIST_KEY: &[u8] = &[
    'i' as u8, 'd' as u8, '_' as u8, 'l' as u8, 'i' as u8, 's' as u8, 't' as u8,
];

pub type KeyList = Vec<Vec<u8>>;

pub fn get_kv_key(collection: &[u8], key: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(collection);
    result.extend_from_slice(SEPARATORS);
    result.extend_from_slice(key);
    result
}

fn get_id_list_key(grouping: &[u8]) -> Vec<u8> {
    get_kv_key(grouping, ID_LIST_KEY)
}

pub fn get_id_list(grouping: &[u8], core: &mut UnumCore) -> Vec<Vec<u8>> {
    let id_list_key = get_id_list_key(grouping);
    let key_list = {
        let get_key_list = AtomicGetInstruction {
            targets: vec![GetTargetSpec {
                key: id_list_key.clone(),
                height: None,
            }],
        };
        match core.execute(&Instruction::AtomicGet(get_key_list)) {
            Ok(Answer::GetOk(get_list_answer)) => {
                match deserialize::<KeyList>(&get_list_answer.items[0]) {
                    Err(_error) => vec![],
                    Ok(key_list) => key_list,
                }
            }
            _ => vec![],
        }
    };
    key_list
}

pub fn set_id_list(grouping: &[u8], core: &mut UnumCore, id_list: &[Vec<u8>]) -> UnumResult<()> {
    let id_list_key = get_id_list_key(grouping);
    match serialize(&id_list) {
        Err(_error) => return Err(ExecutorError::CannotSerialize.into()),
        Ok(data) => {
            let update_key_list = AtomicSetInstruction {
                targets: vec![SetTargetSpec {
                    key: id_list_key.clone(),
                    value: data,
                }],
            };
            match core.execute(&Instruction::AtomicSet(update_key_list)) {
                Err(error) => Err(error),
                Ok(_) => Ok(()),
            }
        }
    }
}

#[cfg(test)]
mod executor_shared_functions_test {
    use crate::executor::shared::{get_id_list, get_id_list_key, get_kv_key, set_id_list};
    use crate::storage::core::UnumCore;
    use crate::storage::kv::KeyValueEngine;

    #[test]
    fn test_get_kv_key() {
        let collection = "collection";
        let id = "id";
        let kv_key = get_kv_key(collection.as_bytes(), id.as_bytes());
        assert_eq!(kv_key, "collection/id".as_bytes());
    }

    #[test]
    fn test_get_id_list_key() {
        let collection = "collection";
        let kv_key = get_id_list_key(collection.as_bytes());
        assert_eq!(kv_key, "collection/id_list".as_bytes());
    }

    #[test]
    fn test_id_list() {
        let chain_name = "chain";
        match UnumCore::new(&KeyValueEngine::HashMap, chain_name.as_bytes()) {
            Err(_error) => panic!("Cannot initialize core"),
            Ok(mut core) => {
                let collection = "collection".as_bytes();
                let input_list: Vec<Vec<u8>> = vec![
                    vec![0],
                    vec![1, 2, 3, 4, 5, 6],
                    "collection/yet-another-title".as_bytes().to_vec(),
                ];
                set_id_list(collection, &mut core, &input_list);
                let output_list = get_id_list(collection, &mut core);
                assert_eq!(input_list, output_list);
            }
        }
    }
}
