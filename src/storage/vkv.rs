/*
 *  Versioned key-value store
**/

use std::cmp::Ordering;

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use crate::config::UNUM_VERSION;
use crate::declarations::errors::{UnumError, UnumResult};
use crate::declarations::instructions::{
    Answer, GetOkAnswer, Instruction, ReadNamespaceOkAnswer, RevertAllOkAnswer, RevertOkAnswer,
    SetOkAnswer, SwitchNamespaceOkAnswer,
};
use crate::storage::kv::hashmap::HashMapStore;
use crate::storage::kv::redis::RedisStore;
use crate::storage::kv::rocks::RocksStore;
use crate::storage::kv::KeyValueEngine;
use crate::storage::kv::KeyValueStore;
use crate::utils::{u64_to_u8_array, u8_array_to_u64};

pub type InstructionHeight = u64;

#[derive(Debug)]
pub enum VkvError {
    CannotGetEntry,
    CannotSerializeEntry,
}

#[repr(u8)]
enum KeyPrefix {
    StandAlone = 'x' as u8,
    HeightToInstruction = 'i' as u8,
    HeightToInstructionMeta = 'm' as u8,
    KeyToEntry = 'e' as u8,
}

const COMMIT_HEIGHT_KEY: &str = "commit-height";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UpdateRecord {
    height: InstructionHeight,
    value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Entry {
    api_version: u32,
    pub updates: Vec<UpdateRecord>,
}

pub struct UnumVersionedKeyValueStore {
    pub kv_engine: Box<KeyValueStore>,
}

impl UnumVersionedKeyValueStore {
    pub fn new(
        engine_choice: &KeyValueEngine,
        namespace: &[u8],
    ) -> Result<UnumVersionedKeyValueStore, UnumError> {
        let engine: Box<KeyValueStore> = match engine_choice {
            KeyValueEngine::Redis => Box::new(RedisStore::new(namespace)?),
            KeyValueEngine::HashMap => Box::new(HashMapStore::new(namespace)),
            KeyValueEngine::Rocks => Box::new(RocksStore::new(namespace)?),
        };
        let store = UnumVersionedKeyValueStore { kv_engine: engine };
        Ok(store)
    }

    fn get_with_key_prefix(&self, key_prefix: KeyPrefix, key: &[u8]) -> UnumResult<Vec<u8>> {
        let mut v: Vec<u8> = vec![key_prefix as u8];
        v.extend_from_slice(key);
        self.kv_engine.get(&v)
    }

    fn set_with_key_prefix(
        &mut self,
        key_prefix: KeyPrefix,
        key: &[u8],
        value: &[u8],
    ) -> UnumResult<Vec<u8>> {
        let mut v: Vec<u8> = vec![key_prefix as u8];
        v.extend_from_slice(key);
        self.kv_engine.set(&v, value)
    }

    fn get_height(&self) -> InstructionHeight {
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

    fn set_height(&mut self, height: InstructionHeight) -> UnumResult<Vec<u8>> {
        self.set_with_key_prefix(
            KeyPrefix::StandAlone,
            COMMIT_HEIGHT_KEY.as_bytes(),
            &u64_to_u8_array(height),
        )
    }

    fn save_instruction_by_height(
        &mut self,
        height: InstructionHeight,
        instruction: &Instruction,
    ) -> UnumResult<Vec<u8>> {
        match serialize(instruction) {
            Err(_error) => Err(UnumError::SerializationFail),
            Ok(serialized_instruction) => self.set_with_key_prefix(
                KeyPrefix::HeightToInstruction,
                &u64_to_u8_array(height),
                &serialized_instruction,
            ),
        }
    }

    fn get_instruction_by_height(&self, height: InstructionHeight) -> UnumResult<Instruction> {
        match self.get_with_key_prefix(KeyPrefix::HeightToInstruction, &u64_to_u8_array(height)) {
            Err(_error) => Err(UnumError::ReadError),
            Ok(instruction_bytes) => match deserialize::<Instruction>(&instruction_bytes) {
                Err(_error) => Err(UnumError::DeserializationFail),
                Ok(instruction) => Ok(instruction),
            },
        }
    }

    fn save_instruction_meta_by_height(
        &mut self,
        height: InstructionHeight,
        instruction_meta: &InstructionMeta,
    ) -> UnumResult<Vec<u8>> {
        match serialize(instruction_meta) {
            Err(_error) => Err(UnumError::SerializationFail),
            Ok(serialized_instruction) => self.set_with_key_prefix(
                KeyPrefix::HeightToInstructionMeta,
                &u64_to_u8_array(height),
                &serialized_instruction,
            ),
        }
    }

    fn get_instruction_meta_by_height(
        &self,
        height: InstructionHeight,
    ) -> UnumResult<InstructionMeta> {
        match self.get_with_key_prefix(KeyPrefix::HeightToInstructionMeta, &u64_to_u8_array(height))
        {
            Err(_error) => Err(UnumError::ReadError),
            Ok(meta_bytes) => match deserialize::<InstructionMeta>(&meta_bytes) {
                Err(_error) => Err(UnumError::DeserializationFail),
                Ok(meta) => Ok(meta),
            },
        }
    }

    fn get_entry(&self, key: &[u8]) -> UnumResult<Entry> {
        let meta_bytes = self.get_with_key_prefix(KeyPrefix::KeyToEntry, key);
        match meta_bytes {
            Err(_error) => {
                return Err(VkvError::CannotGetEntry.into());
            }
            Ok(meta_bytes) => {
                let deserialized = deserialize::<Entry>(&meta_bytes);
                if let Ok(meta) = deserialized {
                    return Ok(meta);
                } else {
                    return Err(VkvError::CannotSerializeEntry.into());
                }
            }
        }
    }

    fn execute_set(
        &mut self,
        key: &[u8],
        value: &[u8],
        height: InstructionHeight,
    ) -> UnumResult<Vec<u8>> {
        match self.get_entry(key) {
            Err(_err) => {
                let first_entry = UpdateRecord {
                    height,
                    value: value.to_vec(),
                };

                let new_meta = Entry {
                    api_version: UNUM_VERSION,
                    updates: vec![first_entry],
                };
                match serialize(&new_meta) {
                    Err(_error) => Err(UnumError::SerializationFail),
                    Ok(serialized_meta) => {
                        println!(
                            "Saving new update on key {:?}",
                            String::from_utf8_lossy(key)
                        );
                        return self.set_with_key_prefix(
                            KeyPrefix::KeyToEntry,
                            key,
                            &serialized_meta,
                        );
                    }
                }
            }
            Ok(mut existing_entry) => {
                let new_record = UpdateRecord {
                    height,
                    value: value.to_vec(),
                };
                existing_entry.updates.push(new_record);
                self.save_entry_by_key(key, &existing_entry)
            }
        }
    }

    fn save_entry_by_key(&mut self, primary_key: &[u8], new_entry: &Entry) -> UnumResult<Vec<u8>> {
        let serialized = serialize(new_entry);
        match serialized {
            Err(_error) => Err(UnumError::SerializationFail),
            Ok(serialized_meta) => {
                println!(
                    "Updating existing index on key {:?}",
                    String::from_utf8_lossy(primary_key)
                );
                return self.set_with_key_prefix(
                    KeyPrefix::KeyToEntry,
                    primary_key,
                    &serialized_meta,
                );
            }
        }
    }

    pub fn get_at_height(
        &mut self,
        key: &[u8],
        target_height: InstructionHeight,
    ) -> UnumResult<Vec<u8>> {
        match self.get_entry(key) {
            Err(error) => Err(error),
            Ok(index) => {
                for record in index.updates.iter().rev() {
                    if record.height <= target_height {
                        return Ok(record.value.clone());
                    }
                }
                return Err(UnumError::ReadError);
            }
        }
    }
    pub fn revert_one(
        &mut self,
        primary_key: &[u8],
        target_height: InstructionHeight,
        next_height: InstructionHeight,
    ) -> UnumResult<Vec<u8>> {
        match self.get_with_key_prefix(KeyPrefix::KeyToEntry, primary_key) {
            Err(_error) => Err(UnumError::WriteError),
            Ok(meta_bytes) => match deserialize::<Entry>(&meta_bytes) {
                Err(_error) => Err(UnumError::WriteError),
                Ok(meta) => {
                    let mut target_update_index = 0;
                    let mut found = false;
                    while !found && target_update_index < meta.updates.len() - 1 {
                        let this_update = &meta.updates[target_update_index];
                        let next_update = &meta.updates[target_update_index + 1];
                        if this_update.height <= target_height && target_height < next_update.height
                        {
                            found = true;
                        } else {
                            target_update_index += 1;
                        }
                    }
                    if found {
                        let mut new_meta = meta.clone();
                        let mut new_record = meta.updates[target_update_index].clone();
                        new_record.height = next_height;
                        new_meta.updates.push(new_record);
                        return self.save_entry_by_key(primary_key, &new_meta);
                    } else {
                        Err(UnumError::WriteError)
                    }
                }
            },
        }
    }

    pub fn increment_height(&mut self) -> Result<InstructionHeight, UnumError> {
        let height = self.get_height() + 1;
        match self.set_height(height) {
            Err(error) => return Err(error),
            Ok(_) => {}
        }
        return Ok(height);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RevertAllInstructionMeta {
    affected_keys: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum InstructionMeta {
    RevertAll(RevertAllInstructionMeta),
}

pub trait VersionedKeyValueStore {
    fn get_current_height(&self) -> InstructionHeight;
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, UnumError>;
}

fn byte_array_compare(vec_a: &[u8], vec_b: &[u8]) -> Ordering {
    let len_a = vec_a.len();
    let len_b = vec_b.len();
    if len_a < len_b {
        return Ordering::Less;
    } else if len_b > len_a {
        return Ordering::Greater;
    } else {
        let mut i = 0;
        while i < len_a {
            if vec_a[i] < vec_b[i] {
                return Ordering::Less;
            } else if vec_a[i] > vec_b[i] {
                return Ordering::Greater;
            }
            i += 1;
        }
    }
    return Ordering::Equal;
}

fn extract_affected_keys(
    store: &UnumVersionedKeyValueStore,
    target_height: InstructionHeight,
    current_height: InstructionHeight,
) -> Vec<Vec<u8>> {
    let mut affected_keys: Vec<Vec<u8>> = vec![];
    let mut height = current_height;
    while height >= target_height {
        if let Ok(instruction) = store.get_instruction_by_height(height) {
            match instruction {
                Instruction::SwitchNamespace(_switch_namespace) => (),
                Instruction::ReadNamespace(_read_namespace) => (),
                Instruction::Get(_get) => (),
                Instruction::Set(set) => {
                    for target in set.targets {
                        affected_keys.push(target.key)
                    }
                }
                Instruction::Revert(revert) => {
                    for target in revert.targets {
                        affected_keys.push(target.key)
                    }
                }
                Instruction::RevertAll(_revert_all) => {
                    if let Ok(meta) = store.get_instruction_meta_by_height(height) {
                        match meta {
                            InstructionMeta::RevertAll(meta) => {
                                affected_keys.extend_from_slice(&meta.affected_keys)
                            }
                        }
                    }
                }
            }
        }
        height -= 1;
    }
    affected_keys.sort_unstable_by(|a, b| byte_array_compare(a, b));
    affected_keys.dedup_by(|a, b| byte_array_compare(a, b) == Ordering::Equal);
    return affected_keys;
}

impl VersionedKeyValueStore for UnumVersionedKeyValueStore {
    fn get_current_height(&self) -> InstructionHeight {
        return self.get_height();
    }
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, UnumError> {
        match instruction {
            Instruction::Get(get) => {
                let mut results: Vec<Vec<u8>> = Vec::new();
                let base_height = self.get_height();
                for target in get.targets.iter() {
                    let target_height = match target.height {
                        None => base_height,
                        Some(height) => height,
                    };
                    let result = self.get_at_height(&target.key, target_height)?;
                    results.push(result)
                }
                return Ok(Answer::GetOk(GetOkAnswer { items: results }));
            }
            Instruction::Set(set) => {
                let mut results: Vec<Vec<u8>> = Vec::new();
                let height = self.increment_height()?;
                if let Err(_) = self.save_instruction_by_height(height, instruction) {
                    return Err(UnumError::WriteError);
                }
                for target in set.targets.iter() {
                    match self.execute_set(&target.key, &target.value, height) {
                        Err(_error) => return Err(UnumError::WriteError),
                        Ok(result) => results.push(result),
                    }
                }
                return Ok(Answer::SetOk(SetOkAnswer { items: results }));
            }
            Instruction::Revert(revert) => {
                let mut results: Vec<Vec<u8>> = Vec::new();
                let height = self.increment_height()?;
                if let Err(_) = self.save_instruction_by_height(height, instruction) {
                    return Err(UnumError::WriteError);
                }
                for target in revert.targets.iter() {
                    match self.revert_one(&target.key, target.height, height) {
                        Err(_error) => return Err(UnumError::WriteError),
                        Ok(result) => results.push(result),
                    }
                }
                return Ok(Answer::RevertOk(RevertOkAnswer { items: results }));
            }
            Instruction::RevertAll(revert_all) => {
                let height = self.increment_height()?;
                let target_height = revert_all.target_height;
                self.save_instruction_by_height(height, instruction)?;

                // Find affected keys
                let affected_keys = extract_affected_keys(&self, target_height, height);

                for key in affected_keys.iter() {
                    self.revert_one(&key, target_height, height)?;
                }

                // Save affected for later use
                let instruction_meta = InstructionMeta::RevertAll(RevertAllInstructionMeta {
                    affected_keys: affected_keys.clone(),
                });
                self.save_instruction_meta_by_height(height, &instruction_meta)?;

                return Ok(Answer::RevertAllOk(RevertAllOkAnswer {
                    reverted_keys: affected_keys,
                }));
            }
            Instruction::SwitchNamespace(set_namespace) => {
                match self
                    .kv_engine
                    .switch_namespace(&set_namespace.new_namespace)
                {
                    Err(error) => Err(error),
                    Ok(_) => Ok(Answer::SwitchNamespaceOk(SwitchNamespaceOkAnswer {
                        new_namespace: self.kv_engine.read_namespace().to_vec(),
                    })),
                }
            }
            Instruction::ReadNamespace(_get_namespace) => {
                return Ok(Answer::ReadNamespaceOk(ReadNamespaceOkAnswer {
                    namespace: self.kv_engine.read_namespace().to_vec(),
                }))
            }
        }
    }
}
