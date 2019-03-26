/*
 *  Versioned key-value store
**/

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::DB_VERSION;
use crate::errors::UnumError;
use crate::utils::{u64_to_u8_array, u8_array_to_u64};

use crate::storage::kv::hashmap::HashMapStore;
use crate::storage::kv::redis::RedisStore;
use crate::storage::kv::KeyValueEngine;
use crate::storage::kv::KeyValueStore;
use crate::storage::kv::KvResult;

pub type CommitHeight = u64;

#[repr(u8)]
enum KeyPrefix {
    StandAlone = 'x' as u8,
    HashToValue = 'v' as u8,
    KeyToMeta = 'm' as u8,
}

const COMMIT_HEIGHT_KEY: &str = "commit-height";

#[repr(u8)]
enum ValueFormat {
    Raw = 0,
    BSON = 0x10,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CommitRecord {
    commit_height: CommitHeight,
    hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EntryMeta {
    api_version: u8,
    pub commit_records: Vec<CommitRecord>,
}

fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(data);
    hasher.result().to_vec()
}

pub struct UnumVersionedKeyValueStore {
    pub kv_engine: Box<KeyValueStore>,
    pub commit_height: CommitHeight,
}

impl UnumVersionedKeyValueStore {
    pub fn new(engine_choice: KeyValueEngine) -> UnumVersionedKeyValueStore {
        let mut engine: Box<KeyValueStore> = match engine_choice {
            KeyValueEngine::Redis => Box::new(RedisStore {
                redis_client: None,
                redis_connection: None,
            }),
            KeyValueEngine::HashMap => Box::new(HashMapStore { hashmap: None }),
        };
        engine.initialize();

        let mut store = UnumVersionedKeyValueStore {
            kv_engine: engine,
            commit_height: 0,
        };
        store.initialize();
        store
    }

    fn get_with_key_prefix(&self, key_prefix: KeyPrefix, key: &[u8]) -> KvResult<Vec<u8>> {
        let mut v: Vec<u8> = vec![key_prefix as u8];
        v.extend_from_slice(key);
        self.kv_engine.get(&v)
    }

    fn set_with_key_prefix(
        &mut self,
        key_prefix: KeyPrefix,
        key: &[u8],
        value: &[u8],
    ) -> KvResult<Vec<u8>> {
        let mut v: Vec<u8> = vec![key_prefix as u8];
        v.extend_from_slice(key);
        self.kv_engine.set(&v, value)
    }

    fn get_commit_height(&mut self) -> CommitHeight {
        let height = self.get_with_key_prefix(KeyPrefix::StandAlone, COMMIT_HEIGHT_KEY.as_bytes());
        match height {
            Err(_error) => 0,
            Ok(height) => {
                if height.len() < 8 {
                    println!("Unexpected height data len {}", height.len());
                    return 0;
                }
                u8_array_to_u64(&[
                    height[0], height[1], height[2], height[3], height[4], height[5], height[6],
                    height[7],
                ])
            }
        }
    }

    fn save_commit_height(&mut self) -> KvResult<Vec<u8>> {
        self.set_with_key_prefix(
            KeyPrefix::StandAlone,
            COMMIT_HEIGHT_KEY.as_bytes(),
            &u64_to_u8_array(self.commit_height),
        )
    }

    fn save_value_by_hash(&mut self, value: &[u8]) -> KvResult<Vec<u8>> {
        self.set_with_key_prefix(KeyPrefix::HashToValue, &sha256(value), value)
    }

    fn get_value_by_hash(&self, hash: Vec<u8>) -> KvResult<Vec<u8>> {
        self.get_with_key_prefix(KeyPrefix::HashToValue, &hash)
    }

    fn get_meta_by_key(&self, key: &[u8]) -> Option<EntryMeta> {
        let meta_bytes = self.get_with_key_prefix(KeyPrefix::KeyToMeta, key);
        match meta_bytes {
            Err(_error) => {
                return None;
            }
            Ok(meta_bytes) => {
                let deserialized = deserialize::<EntryMeta>(&meta_bytes);
                if let Ok(meta) = deserialized {
                    return Some(meta);
                } else {
                    return None;
                }
            }
        }
    }

    fn commit_set(&mut self, key: &[u8], value: &[u8]) -> KvResult<Vec<u8>> {
        match self.get_meta_by_key(key) {
            None => {
                let first_commit_record = CommitRecord {
                    commit_height: self.get_commit_height(),
                    hash: sha256(value).to_vec(),
                };

                let new_meta = EntryMeta {
                    api_version: DB_VERSION,
                    commit_records: vec![first_commit_record],
                };
                match serialize(&new_meta) {
                    Err(_error) => Err(UnumError::SerializationFail),
                    Ok(serialized_meta) => {
                        println!("Saving new index on key {:?}", String::from_utf8_lossy(key));
                        return self.set_with_key_prefix(
                            KeyPrefix::KeyToMeta,
                            key,
                            &serialized_meta,
                        );
                    }
                }
            }
            Some(mut existing_meta) => {
                let new_record = CommitRecord {
                    commit_height: self.commit_height,
                    hash: sha256(value),
                };
                existing_meta.commit_records.push(new_record);
                self.save_meta_key_primary_key(key, &existing_meta)
            }
        }
    }

    fn get_all_keys_by_prefix(&mut self, prefix: KeyPrefix) -> Vec<Vec<u8>> {
        let pattern = format!("{}{}", prefix as u8 as char, "*");
        match self.kv_engine.keys(&pattern) {
            Err(_error) => return vec![],
            Ok(keys) => keys,
        }
    }

    fn save_meta_key_primary_key(
        &mut self,
        primary_key: &[u8],
        new_meta: &EntryMeta,
    ) -> KvResult<Vec<u8>> {
        let serialized = serialize(new_meta);
        match serialized {
            Err(_error) => Err(UnumError::SerializationFail),
            Ok(serialized_meta) => {
                println!(
                    "Updating existing index on key {:?}",
                    String::from_utf8_lossy(primary_key)
                );
                return self.set_with_key_prefix(
                    KeyPrefix::KeyToMeta,
                    primary_key,
                    &serialized_meta,
                );
            }
        }
    }
}

impl KeyValueStore for UnumVersionedKeyValueStore {
    fn initialize(&mut self) -> KvResult<()> {
        self.commit_height = self.get_commit_height();
        Ok(())
    }

    fn get(&self, key: &[u8]) -> KvResult<Vec<u8>> {
        let index = self.get_meta_by_key(key);
        match index {
            None => Err(UnumError::ReadError),
            Some(index) => {
                let last_record = index.commit_records.last();
                match last_record {
                    None => Err(UnumError::ReadError),
                    Some(last_record) => {
                        let value = self.get_value_by_hash(last_record.hash.clone())?;
                        let response = Ok(value);
                        return response;
                    }
                }
            }
        }
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> KvResult<Vec<u8>> {
        self.commit_height += 1;
        self.save_commit_height()?;
        self.save_value_by_hash(value)?;
        self.commit_set(key, value)
    }

    fn keys(&self, pattern: &str) -> KvResult<Vec<Vec<u8>>> {
        return self.kv_engine.keys(pattern);
    }
}

pub trait VersionedKeyValueStore: KeyValueStore {
    fn get_at_version_number(&mut self, key: &[u8], version: CommitHeight) -> KvResult<Vec<u8>>;
    fn get_latest_version_number(&self) -> CommitHeight;
    fn revert_one(&mut self, extern_key: &[u8], target_height: CommitHeight) -> KvResult<Vec<u8>>;
    fn revert_all(&mut self, new_version: CommitHeight) -> KvResult<Vec<u8>>;
}

impl VersionedKeyValueStore for UnumVersionedKeyValueStore {
    fn get_at_version_number(
        &mut self,
        key: &[u8],
        target_height: CommitHeight,
    ) -> KvResult<Vec<u8>> {
        match self.get_meta_by_key(key) {
            None => Err(UnumError::ReadError),
            Some(index) => {
                for record in index.commit_records.iter().rev() {
                    if record.commit_height <= target_height {
                        return self.get_value_by_hash(record.hash.clone());
                    }
                }
                return Err(UnumError::ReadError);
            }
        }
    }
    fn get_latest_version_number(&self) -> CommitHeight {
        self.commit_height
    }
    fn revert_one(&mut self, primary_key: &[u8], target_height: CommitHeight) -> KvResult<Vec<u8>> {
        match self.get_with_key_prefix(KeyPrefix::KeyToMeta, primary_key) {
            Err(_error) => Err(UnumError::WriteError),
            Ok(meta_bytes) => match deserialize::<EntryMeta>(&meta_bytes) {
                Err(_error) => Err(UnumError::WriteError),
                Ok(meta) => {
                    let mut last_valid_record_index = 0;
                    for record in meta.commit_records.iter() {
                        if record.commit_height > target_height {
                            break;
                        }
                        last_valid_record_index += 1;
                    }
                    if last_valid_record_index < meta.commit_records.len() - 1 {
                        let mut new_meta = meta.clone();
                        let mut new_record = meta.commit_records[last_valid_record_index].clone();
                        new_record.commit_height = self.commit_height;
                        new_meta.commit_records.push(new_record);
                        return self.save_meta_key_primary_key(primary_key, &new_meta);
                    } else {
                        Ok(vec![])
                    }
                }
            },
        }
    }
    fn revert_all(&mut self, target_height: CommitHeight) -> KvResult<Vec<u8>> {
        self.commit_height += 1;
        self.save_commit_height()?;
        let meta_keys = self.get_all_keys_by_prefix(KeyPrefix::KeyToMeta);
        for key in meta_keys.iter() {
            self.revert_one(&key[1..], target_height)?;
        }
        Ok(vec![])
    }
}
