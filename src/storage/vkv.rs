/*
 *  Versioned key-value store
**/

use std::cmp::Ordering;

use bincode::{deserialize, serialize, Error as BincodeError};
use serde::{Deserialize, Serialize};

use crate::config::IMMUXDB_VERSION;

use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{
    Answer, GetOkAnswer, GetOneOkAnswer, Instruction, ReadNamespaceOkAnswer, RevertAllOkAnswer,
    RevertOkAnswer, SetOkAnswer, SwitchNamespaceOkAnswer,
};
use crate::storage::kv::hashmap::HashMapStore;
use crate::storage::kv::rocks::RocksStore;
use crate::storage::kv::KeyValueEngine;
use crate::storage::kv::KeyValueStore;

use crate::utils::{u64_to_u8_array, u8_array_to_u64};

pub type InstructionHeight = u64;

#[derive(Debug)]
pub enum VkvError {
    CannotSerializeEntry,
    CannotSerializeInstructionMeta(BincodeError),
    UnexpectedInstruction,
    DeserializationFail,
    GetInstructionRecordFail,
    GetInstructionMetaFail,
    GetHeightFail,
    SuitableRevertVersionNotFound,
    SaveInstructionRecordFail,
}

#[repr(u8)]
enum KeyPrefix {
    StandAlone = 'x' as u8,
    HeightToInstruction = 'i' as u8,
    HeightToInstructionMeta = 'm' as u8,
    KeyToEntry = 'e' as u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InstructionRecord {
    pub instruction: Instruction,
    pub deleted: bool,
}

const COMMIT_HEIGHT_KEY: &str = "commit-height";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UpdateRecord {
    deleted: bool,
    height: InstructionHeight,
    value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Entry {
    api_version: u32,
    pub updates: Vec<UpdateRecord>,
}

pub struct ImmuxDBVersionedKeyValueStore {
    pub kv_engine: Box<KeyValueStore>,
}

impl ImmuxDBVersionedKeyValueStore {
    pub fn new(
        engine_choice: &KeyValueEngine,
        namespace: &[u8],
    ) -> Result<ImmuxDBVersionedKeyValueStore, ImmuxError> {
        let engine: Box<KeyValueStore> = match engine_choice {
            KeyValueEngine::HashMap => Box::new(HashMapStore::new(namespace)),
            KeyValueEngine::Rocks => Box::new(RocksStore::new(namespace)?),
        };
        let store = ImmuxDBVersionedKeyValueStore { kv_engine: engine };
        Ok(store)
    }

    fn get_with_key_prefix(&self, key_prefix: KeyPrefix, key: &[u8]) -> ImmuxResult<Vec<u8>> {
        let mut v: Vec<u8> = vec![key_prefix as u8];
        v.extend_from_slice(key);
        self.kv_engine.get(&v)
    }

    fn set_with_key_prefix(
        &mut self,
        key_prefix: KeyPrefix,
        key: &[u8],
        value: &[u8],
    ) -> ImmuxResult<Vec<u8>> {
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

    pub fn set_height(&mut self, height: InstructionHeight) -> ImmuxResult<Vec<u8>> {
        self.set_with_key_prefix(
            KeyPrefix::StandAlone,
            COMMIT_HEIGHT_KEY.as_bytes(),
            &u64_to_u8_array(height),
        )
    }

    fn save_instruction_record_by_height(
        &mut self,
        height: InstructionHeight,
        instruction_record: &InstructionRecord,
    ) -> ImmuxResult<Vec<u8>> {
        match serialize(instruction_record) {
            Err(_error) => Err(VkvError::CannotSerializeEntry.into()),
            Ok(serialized_instruction_record) => self.set_with_key_prefix(
                KeyPrefix::HeightToInstruction,
                &u64_to_u8_array(height),
                &serialized_instruction_record,
            ),
        }
    }

    fn get_instruction_record_by_height(
        &self,
        height: InstructionHeight,
    ) -> ImmuxResult<InstructionRecord> {
        match self.get_with_key_prefix(KeyPrefix::HeightToInstruction, &u64_to_u8_array(height)) {
            Err(_error) => Err(VkvError::GetInstructionRecordFail.into()),
            Ok(instruction_record_bytes) => {
                match deserialize::<InstructionRecord>(&instruction_record_bytes) {
                    Err(_error) => Err(VkvError::DeserializationFail.into()),
                    Ok(instruction_record) => Ok(instruction_record),
                }
            }
        }
    }

    pub fn invalidate_instruction_record_after_height(
        &mut self,
        target_height: InstructionHeight,
    ) -> ImmuxResult<()> {
        let mut height = self.get_height();
        while height > target_height {
            let mut instruction_record = self.get_instruction_record_by_height(height)?;
            instruction_record.deleted = true;
            self.save_instruction_record_by_height(height, &instruction_record)?;
            height -= 1;
        }
        return Ok(());
    }

    fn save_instruction_meta_by_height(
        &mut self,
        height: InstructionHeight,
        instruction_meta: &InstructionMeta,
    ) -> ImmuxResult<Vec<u8>> {
        match serialize(instruction_meta) {
            Err(_error) => Err(VkvError::CannotSerializeInstructionMeta(_error).into()),
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
    ) -> ImmuxResult<InstructionMeta> {
        match self.get_with_key_prefix(KeyPrefix::HeightToInstructionMeta, &u64_to_u8_array(height))
        {
            Err(_error) => Err(VkvError::GetInstructionMetaFail.into()),
            Ok(meta_bytes) => match deserialize::<InstructionMeta>(&meta_bytes) {
                Err(_error) => Err(VkvError::DeserializationFail.into()),
                Ok(meta) => Ok(meta),
            },
        }
    }

    pub fn invalidate_instruction_meta_after_height(
        &mut self,
        target_height: InstructionHeight,
    ) -> ImmuxResult<()> {
        let mut height = self.get_height();
        while height >= target_height {
            match self.get_instruction_meta_by_height(height) {
                Err(_error) => (),
                Ok(meta) => match meta {
                    InstructionMeta::RevertAll(mut revert_all_instruction_meta) => {
                        revert_all_instruction_meta.deleted = true;
                        self.save_instruction_meta_by_height(
                            height,
                            &InstructionMeta::RevertAll(revert_all_instruction_meta),
                        )?;
                    }
                },
            }
            if height == 0 {
                break;
            } else {
                height -= 1;
            }
        }
        return Ok(());
    }

    fn get_entry(&self, key: &[u8]) -> ImmuxResult<Entry> {
        let meta_bytes = self.get_with_key_prefix(KeyPrefix::KeyToEntry, key);
        match meta_bytes {
            Err(error) => {
                return Err(error);
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
    ) -> ImmuxResult<Vec<u8>> {
        match self.get_entry(key) {
            Err(_err) => {
                let first_entry = UpdateRecord {
                    height,
                    value: value.to_vec(),
                    deleted: false,
                };

                let new_meta = Entry {
                    api_version: IMMUXDB_VERSION,
                    updates: vec![first_entry],
                };
                match serialize(&new_meta) {
                    Err(error) => Err(VkvError::CannotSerializeInstructionMeta(error).into()),
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
                    deleted: false,
                };
                existing_entry.updates.push(new_record);
                self.save_entry_by_key(key, &existing_entry)
            }
        }
    }

    fn save_entry_by_key(&mut self, primary_key: &[u8], new_entry: &Entry) -> ImmuxResult<Vec<u8>> {
        let serialized = serialize(new_entry);
        match serialized {
            Err(_error) => Err(VkvError::CannotSerializeEntry.into()),
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

    pub fn invalidate_update_after_height(
        &mut self,
        primary_key: &[u8],
        target_height: InstructionHeight,
    ) -> ImmuxResult<Vec<u8>> {
        match self.get_entry(primary_key) {
            Err(error) => Err(error),
            Ok(mut existing_entry) => {
                for update in existing_entry.updates.iter_mut().rev() {
                    if update.height > target_height {
                        update.deleted = true;
                    } else {
                        break;
                    }
                }
                self.save_entry_by_key(primary_key, &existing_entry)
            }
        }
    }

    pub fn get_at_height(
        &mut self,
        key: &[u8],
        target_height: InstructionHeight,
    ) -> ImmuxResult<Vec<u8>> {
        match self.get_entry(key) {
            Err(error) => Err(error),
            Ok(index) => {
                for record in index.updates.iter().rev() {
                    if record.height <= target_height {
                        return Ok(record.value.clone());
                    }
                }
                return Err(VkvError::GetHeightFail.into());
            }
        }
    }

    pub fn revert_one(
        &mut self,
        primary_key: &[u8],
        target_height: InstructionHeight,
        next_height: InstructionHeight,
    ) -> ImmuxResult<Vec<u8>> {
        match self.get_with_key_prefix(KeyPrefix::KeyToEntry, primary_key) {
            Err(error) => Err(error),
            Ok(meta_bytes) => match deserialize::<Entry>(&meta_bytes) {
                Err(_error) => Err(VkvError::DeserializationFail.into()),
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
                        Err(VkvError::SuitableRevertVersionNotFound.into())
                    }
                }
            },
        }
    }

    pub fn increment_height(&mut self) -> Result<InstructionHeight, ImmuxError> {
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
    deleted: bool,
    affected_keys: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum InstructionMeta {
    RevertAll(RevertAllInstructionMeta),
}

pub trait VersionedKeyValueStore {
    fn get_current_height(&self) -> InstructionHeight;
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, ImmuxError>;
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

pub fn extract_affected_keys(
    store: &ImmuxDBVersionedKeyValueStore,
    target_height: InstructionHeight,
    current_height: InstructionHeight,
) -> ImmuxResult<Vec<Vec<u8>>> {
    let mut affected_keys: Vec<Vec<u8>> = vec![];
    let mut height = current_height;
    while height >= target_height {
        if let Ok(instruction_record) = store.get_instruction_record_by_height(height) {
            match instruction_record.instruction {
                Instruction::SwitchNamespace(_switch_namespace) => (),
                Instruction::ReadNamespace(_read_namespace) => (),
                Instruction::AtomicGet(_get) => (),
                Instruction::AtomicSet(set) => {
                    for target in set.targets {
                        affected_keys.push(target.key)
                    }
                }
                Instruction::AtomicRevert(revert) => {
                    for target in revert.targets {
                        affected_keys.push(target.key)
                    }
                }
                Instruction::AtomicRevertAll(_revert_all) => {
                    if let Ok(meta) = store.get_instruction_meta_by_height(height) {
                        match meta {
                            InstructionMeta::RevertAll(meta) => {
                                affected_keys.extend_from_slice(&meta.affected_keys)
                            }
                        }
                    }
                }
                _ => {
                    return Err(ImmuxError::VKV(VkvError::UnexpectedInstruction));
                }
            }
        }
        height -= 1;
    }
    affected_keys.sort_unstable_by(|a, b| byte_array_compare(a, b));
    affected_keys.dedup_by(|a, b| byte_array_compare(a, b) == Ordering::Equal);
    return Ok(affected_keys);
}

impl VersionedKeyValueStore for ImmuxDBVersionedKeyValueStore {
    fn get_current_height(&self) -> InstructionHeight {
        return self.get_height();
    }
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, ImmuxError> {
        match instruction {
            Instruction::AtomicGet(get) => {
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
            Instruction::AtomicGetOne(get_one) => {
                let target = &get_one.target;
                let base_height = self.get_height();
                let target_height = match target.height {
                    None => base_height,
                    Some(height) => height,
                };
                let result = self.get_at_height(&target.key, target_height)?;
                return Ok(Answer::GetOneOk(GetOneOkAnswer { item: result }));
            }
            Instruction::AtomicSet(set) => {
                let mut results: Vec<Vec<u8>> = Vec::new();
                let height = self.increment_height()?;
                let instruction_record = InstructionRecord {
                    instruction: instruction.clone(),
                    deleted: false,
                };
                if let Err(_) = self.save_instruction_record_by_height(height, &instruction_record)
                {
                    return Err(VkvError::SaveInstructionRecordFail.into());
                }
                for target in set.targets.iter() {
                    match self.execute_set(&target.key, &target.value, height) {
                        Err(error) => return Err(error),
                        Ok(result) => results.push(result),
                    }
                }
                return Ok(Answer::SetOk(SetOkAnswer { items: results }));
            }
            Instruction::AtomicRevert(revert) => {
                let mut results: Vec<Vec<u8>> = Vec::new();
                let height = self.increment_height()?;
                let instruction_record = InstructionRecord {
                    instruction: instruction.clone(),
                    deleted: false,
                };
                if let Err(_) = self.save_instruction_record_by_height(height, &instruction_record)
                {
                    return Err(VkvError::SaveInstructionRecordFail.into());
                }
                for target in revert.targets.iter() {
                    match self.revert_one(&target.key, target.height, height) {
                        Err(error) => return Err(error),
                        Ok(result) => results.push(result),
                    }
                }
                return Ok(Answer::RevertOk(RevertOkAnswer { items: results }));
            }
            Instruction::AtomicRevertAll(revert_all) => {
                let height = self.increment_height()?;
                let target_height = revert_all.target_height;
                let instruction_record = InstructionRecord {
                    instruction: instruction.clone(),
                    deleted: false,
                };
                self.save_instruction_record_by_height(height, &instruction_record)?;

                // Find affected keys
                let affected_keys = extract_affected_keys(&self, target_height, height)?;

                for key in affected_keys.iter() {
                    self.revert_one(&key, target_height, height)?;
                }

                // Save affected for later use
                let instruction_meta = InstructionMeta::RevertAll(RevertAllInstructionMeta {
                    deleted: false,
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
                }));
            }
            _ => {
                return Err(ImmuxError::VKV(VkvError::UnexpectedInstruction));
            }
        }
    }
}
