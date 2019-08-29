/*
 *  Versioned key-value store
**/

use std::cmp::Ordering;

use bincode::{deserialize, serialize, Error as BincodeError};
use serde::{Deserialize, Serialize};

use crate::config::{KVKeySigil, IMMUXDB_VERSION};

use crate::declarations::basics::{BoxedStoreKey, BoxedStoreValue, StoreKey, StoreValue};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::storage::instructions::{
    Answer, DBSystemAnswer, DBSystemInstruction, DataAnswer, DataInstruction, DataReadAnswer,
    DataReadInstruction, DataWriteAnswer, DataWriteInstruction, GetJournalOkAnswer,
    GetManyOkAnswer, GetManyTargetSpec, GetOneOkAnswer, Instruction, ReadNamespaceOkAnswer,
    RevertAllOkAnswer, RevertOkAnswer, SetOkAnswer, StoreNamespace, SwitchNamespaceOkAnswer,
};
use crate::storage::kv::{
    HashMapStore, KVKey, KVKeySegment, KVNamespace, KVValue, KeyValueEngine, KeyValueStore,
    RocksStore,
};
use crate::storage::vkv::chain_height::ChainHeight;
use crate::storage::vkv::journal::{UnitJournal, UpdateRecord};

#[derive(Debug)]
pub enum VkvError {
    CannotSerializeJournal,
    CannotSerializeInstructionMeta(BincodeError),
    UnexpectedInstruction,
    DeserializationFail,
    GetInstructionRecordFail,
    GetInstructionMetaFail,
    GetHeightFail,
    SuitableRevertVersionNotFound,
    SaveInstructionRecordFail,
    UpdateRecordParsing,
    JournalParsing,
    EmptyUpdatesInJournal,
}

fn prefix_extractor(key: &[u8]) -> &[u8] {
    if key.len() <= 1 {
        return &[];
    }
    if key[0] == KVKeySigil::ChainInfo as u8 {
        return &key[0..1];
    } else if key[0] == KVKeySigil::HeightToInstructionRecord as u8 {
        return &key[0..1];
    } else if key[0] == KVKeySigil::HeightToInstructionMeta as u8 {
        return &key[0..1];
    } else if key[0] == KVKeySigil::UnitJournal as u8 {
        let grouping_name_length = key[1];
        let prefix_length = (1 + 1 + grouping_name_length) as usize;
        return &key[0..prefix_length];
    } else {
        return &[];
    }
}

fn get_journal_kvkey(store_key: &StoreKey) -> KVKey {
    let mut journal_key_bytes = Vec::new();
    journal_key_bytes.push(KVKeySigil::UnitJournal as u8);
    journal_key_bytes.extend_from_slice(store_key.as_slice());
    return journal_key_bytes.into();
}

fn extract_journal_store_key(key: &KVKey) -> StoreKey {
    StoreKey::new(&key.as_bytes()[1..])
}

fn get_chain_height_kvkey() -> KVKey {
    KVKey::from(vec![KVKeySigil::ChainHeight as u8])
}

fn get_instruction_record_kvkey(height: ChainHeight) -> KVKey {
    let mut result = Vec::new();
    result.push(KVKeySigil::HeightToInstructionRecord as u8);
    let height_bytes: Vec<u8> = height.into();
    result.extend(height_bytes);
    result.into()
}

fn get_instruction_meta_kvkey(height: ChainHeight) -> KVKey {
    let mut result = Vec::new();
    result.push(KVKeySigil::HeightToInstructionMeta as u8);
    let height_bytes: Vec<u8> = height.into();
    result.extend(height_bytes);
    result.into()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InstructionRecord {
    pub api_version: u32,
    pub invalidated: bool,
    pub instruction: Instruction,
}

pub struct ImmuxDBVersionedKeyValueStore {
    pub kv_engine: Box<dyn KeyValueStore>,
}

impl ImmuxDBVersionedKeyValueStore {
    pub fn new(
        engine_choice: &KeyValueEngine,
        data_root: &str,
        namespace: &StoreNamespace,
    ) -> Result<ImmuxDBVersionedKeyValueStore, ImmuxError> {
        let kv_namespace = KVNamespace::from(namespace.to_owned());
        let engine: Box<dyn KeyValueStore> = match engine_choice {
            KeyValueEngine::HashMap => Box::new(HashMapStore::new(&kv_namespace)),
            KeyValueEngine::Rocks => {
                Box::new(RocksStore::new(data_root, &kv_namespace, prefix_extractor)?)
            }
        };
        let store = ImmuxDBVersionedKeyValueStore { kv_engine: engine };
        Ok(store)
    }

    fn get_height(&self) -> ChainHeight {
        match self.kv_engine.get(&get_chain_height_kvkey()) {
            Err(_error) => ChainHeight::new(0),
            Ok(value) => {
                let bytes = value.as_bytes();
                if bytes.len() < 8 {
                    println!("Unexpected height data len {}", bytes.len());
                    ChainHeight::new(0)
                } else {
                    ChainHeight::from([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ])
                }
            }
        }
    }

    pub fn set_height(&mut self, height: ChainHeight) -> ImmuxResult<()> {
        let key = &get_chain_height_kvkey();
        let height_bytes: Vec<u8> = height.into();
        let value = KVValue::from(height_bytes);
        self.kv_engine.set(key, &value)
    }

    fn save_instruction_record_by_height(
        &mut self,
        height: ChainHeight,
        instruction_record: &InstructionRecord,
    ) -> ImmuxResult<()> {
        match serialize(instruction_record) {
            Err(_error) => Err(VkvError::CannotSerializeJournal.into()),
            Ok(serialized_instruction_record) => {
                let key = get_instruction_record_kvkey(height);
                let value: KVValue = serialized_instruction_record.into();
                self.kv_engine.set(&key, &value)?;
                Ok(())
            }
        }
    }

    fn get_instruction_record_by_height(
        &self,
        height: ChainHeight,
    ) -> ImmuxResult<InstructionRecord> {
        let key = get_instruction_record_kvkey(height);
        match self.kv_engine.get(&key) {
            Err(_error) => Err(VkvError::GetInstructionRecordFail.into()),
            Ok(value) => match deserialize::<InstructionRecord>(value.as_bytes()) {
                Err(_error) => Err(VkvError::DeserializationFail.into()),
                Ok(instruction_record) => Ok(instruction_record),
            },
        }
    }

    pub fn invalidate_instruction_record_after_height(
        &mut self,
        target_height: ChainHeight,
    ) -> ImmuxResult<()> {
        let mut height = self.get_height();
        while height > target_height {
            let mut instruction_record = self.get_instruction_record_by_height(height)?;
            instruction_record.invalidated = true;
            self.save_instruction_record_by_height(height, &instruction_record)?;
            height.decrement();
        }
        return Ok(());
    }

    fn save_instruction_meta_by_height(
        &mut self,
        height: ChainHeight,
        instruction_meta: &InstructionMeta,
    ) -> ImmuxResult<()> {
        match serialize(instruction_meta) {
            Err(_error) => Err(VkvError::CannotSerializeInstructionMeta(_error).into()),
            Ok(serialized_instruction) => {
                let key = get_instruction_meta_kvkey(height);
                let value = KVValue::from(serialized_instruction);
                self.kv_engine.set(&key, &value)?;
                Ok(())
            }
        }
    }

    fn get_instruction_meta_by_height(&self, height: ChainHeight) -> ImmuxResult<InstructionMeta> {
        let key = get_instruction_meta_kvkey(height);
        match self.kv_engine.get(&key) {
            Err(_error) => Err(VkvError::GetInstructionMetaFail.into()),
            Ok(value) => match deserialize::<InstructionMeta>(value.as_bytes()) {
                Err(_error) => Err(VkvError::DeserializationFail.into()),
                Ok(meta) => Ok(meta),
            },
        }
    }

    pub fn invalidate_instruction_meta_after_height(
        &mut self,
        target_height: ChainHeight,
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
            if height.is_zero() {
                break;
            } else {
                height.decrement();
            }
        }
        return Ok(());
    }

    fn get_journal(&self, key: &StoreKey) -> ImmuxResult<UnitJournal> {
        let kvkey = get_journal_kvkey(key);
        match self.kv_engine.get(&kvkey) {
            Err(error) => Err(error),
            Ok(value) => match UnitJournal::parse(value.as_bytes()) {
                Err(_) => Err(VkvError::CannotSerializeJournal.into()),
                Ok(journal) => Ok(journal),
            },
        }
    }

    fn execute_versioned_set(
        &mut self,
        store_key: &StoreKey,
        value: &StoreValue,
        height: ChainHeight,
    ) -> ImmuxResult<()> {
        match self.get_journal(store_key) {
            Err(_error) => {
                let first_record = UpdateRecord {
                    height,
                    value: value.to_owned(),
                    deleted: false,
                };
                let new_journal = UnitJournal {
                    api_version: IMMUXDB_VERSION,
                    updates: vec![first_record],
                };
                let kvkey = get_journal_kvkey(store_key);
                let serialized_meta = new_journal.serialize();
                let value: KVValue = serialized_meta.into();
                self.kv_engine.set(&kvkey, &value)?;
                return Ok(());
            }
            Ok(mut existing_journal) => {
                let new_record = UpdateRecord {
                    height,
                    value: value.to_owned(),
                    deleted: false,
                };
                existing_journal.updates.push(new_record);
                let kvkey = get_journal_kvkey(store_key);
                let value: KVValue = existing_journal.serialize().into();
                self.kv_engine.set(&kvkey, &value)?;
                return Ok(());
            }
        }
    }

    pub fn invalidate_update_after_height(
        &mut self,
        key: &StoreKey,
        target_height: ChainHeight,
    ) -> ImmuxResult<()> {
        match self.get_journal(key) {
            Err(error) => Err(error),
            Ok(mut existing_journal) => {
                for update in existing_journal.updates.iter_mut().rev() {
                    if update.height > target_height {
                        update.deleted = true;
                    } else {
                        break;
                    }
                }
                let kvkey = get_journal_kvkey(key);
                let value: KVValue = existing_journal.serialize().into();
                self.kv_engine.set(&kvkey, &value)?;
                return Ok(());
            }
        }
    }

    pub fn get_at_height(
        &mut self,
        key: &StoreKey,
        target_height: ChainHeight,
    ) -> ImmuxResult<StoreValue> {
        match self.get_journal(key) {
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
        key: &StoreKey,
        target_height: ChainHeight,
        next_height: ChainHeight,
    ) -> ImmuxResult<()> {
        let journal_basekey = get_journal_kvkey(key);
        match self.kv_engine.get(&journal_basekey) {
            Err(error) => Err(error),
            Ok(value) => match UnitJournal::parse(value.as_bytes()) {
                Err(_error) => Err(VkvError::DeserializationFail.into()),
                Ok(journal) => {
                    let mut target_update_index = 0;
                    let mut found = false;
                    while !found && target_update_index < journal.updates.len() - 1 {
                        let this_update = &journal.updates[target_update_index];
                        let next_update = &journal.updates[target_update_index + 1];
                        if this_update.height <= target_height && target_height < next_update.height
                        {
                            found = true;
                        } else {
                            target_update_index += 1;
                        }
                    }
                    if found {
                        let mut new_journal = journal.clone();
                        let mut new_record = journal.updates[target_update_index].clone();
                        new_record.height = next_height;
                        new_journal.updates.push(new_record);
                        self.kv_engine
                            .set(&journal_basekey, &new_journal.serialize().into())?;
                        return Ok(());
                    } else {
                        Err(VkvError::SuitableRevertVersionNotFound.into())
                    }
                }
            },
        }
    }

    pub fn increment_chain_height(&mut self) -> Result<ChainHeight, ImmuxError> {
        let mut height = self.get_height();
        height.increment();
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
    affected_keys: Vec<StoreKey>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum InstructionMeta {
    RevertAll(RevertAllInstructionMeta),
}

pub trait VersionedKeyValueStore {
    fn get_current_height(&self) -> ChainHeight;
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, ImmuxError>;
}

fn byte_array_compare(key_a: &StoreKey, key_b: &StoreKey) -> Ordering {
    let vec_a = key_a.as_slice();
    let vec_b = key_b.as_slice();
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
    target_height: ChainHeight,
    current_height: ChainHeight,
) -> ImmuxResult<Vec<StoreKey>> {
    let mut affected_keys: Vec<StoreKey> = vec![];
    let mut height = current_height;
    while height >= target_height {
        if let Ok(instruction_record) = store.get_instruction_record_by_height(height) {
            if instruction_record.invalidated {
                continue;
            }
            match instruction_record.instruction {
                Instruction::DBSystem(_) => (),
                Instruction::TransactionMeta(_) => (),
                Instruction::Data(DataInstruction::Read(_)) => (),
                Instruction::Data(DataInstruction::Write(write_instruction)) => {
                    match write_instruction {
                        DataWriteInstruction::SetMany(set) => {
                            for target in set.targets {
                                affected_keys.push(target.key)
                            }
                        }
                        DataWriteInstruction::RevertMany(revert_many) => {
                            for target in revert_many.targets {
                                affected_keys.push(target.key)
                            }
                        }
                        DataWriteInstruction::RevertAll(_revert_all) => {
                            if let Ok(meta) = store.get_instruction_meta_by_height(height) {
                                match meta {
                                    InstructionMeta::RevertAll(meta) => {
                                        affected_keys.extend_from_slice(&meta.affected_keys)
                                    }
                                }
                            }
                        }
                    };
                }
                _ => {
                    return Err(ImmuxError::VKV(VkvError::UnexpectedInstruction));
                }
            }
        }
        height.decrement();
    }
    affected_keys.sort_unstable_by(|a, b| byte_array_compare(a, b));
    affected_keys.dedup_by(|a, b| byte_array_compare(a, b) == Ordering::Equal);
    return Ok(affected_keys);
}

impl VersionedKeyValueStore for ImmuxDBVersionedKeyValueStore {
    fn get_current_height(&self) -> ChainHeight {
        return self.get_height();
    }
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, ImmuxError> {
        match instruction {
            Instruction::DBSystem(sys_instruction) => match sys_instruction {
                DBSystemInstruction::SwitchNamespace(set_namespace) => {
                    match self
                        .kv_engine
                        .switch_namespace(&set_namespace.new_namespace.to_owned().into())
                    {
                        Err(error) => Err(error),
                        Ok(_) => Ok(Answer::DBSystem(DBSystemAnswer::SwitchNamespaceOk(
                            SwitchNamespaceOkAnswer {
                                new_namespace: self.kv_engine.read_namespace().into(),
                            },
                        ))),
                    }
                }
                DBSystemInstruction::ReadNamespace(_get_namespace) => {
                    return Ok(Answer::DBSystem(DBSystemAnswer::ReadNamespaceOk(
                        ReadNamespaceOkAnswer {
                            namespace: self.kv_engine.read_namespace().into(),
                        },
                    )));
                }
            },

            Instruction::Data(DataInstruction::Read(read_instruction)) => match read_instruction {
                DataReadInstruction::GetMany(get_many) => {
                    let target_height = match get_many.height {
                        None => self.get_height(),
                        Some(height) => height,
                    };
                    match &get_many.targets {
                        GetManyTargetSpec::Keys(keys) => {
                            let mut data: Vec<(BoxedStoreKey, BoxedStoreValue)> =
                                Vec::with_capacity(keys.len());
                            for key in keys {
                                let value = self.get_at_height(&key, target_height)?;
                                data.push((key.to_owned().into(), value.into()))
                            }
                            return Ok(Answer::DataAccess(DataAnswer::Read(
                                DataReadAnswer::GetManyOk(GetManyOkAnswer { data }),
                            )));
                        }
                        GetManyTargetSpec::KeyPrefix(key_prefix) => {
                            let basekey_prefix: KVKeySegment = {
                                let mut result =
                                    Vec::with_capacity(1 + key_prefix.as_slice().len());
                                result.push(KVKeySigil::UnitJournal as u8);
                                result.extend_from_slice(key_prefix.as_slice());
                                result.into()
                            };
                            let base_pairs = self.kv_engine.filter_prefix(&basekey_prefix);
                            let parsed_pairs: Vec<(BoxedStoreKey, Box<UnitJournal>)> = {
                                let mut result = Vec::with_capacity(base_pairs.len());
                                for pair in base_pairs.into_iter() {
                                    // Remove Sigil
                                    let (kvkey, kvvalue) = pair;
                                    let store_key = extract_journal_store_key(&kvkey.into());
                                    let journal = UnitJournal::parse(kvvalue.as_bytes())?;
                                    result.push((store_key.into(), Box::new(journal)));
                                }
                                result
                            };
                            let data: Vec<(BoxedStoreKey, BoxedStoreValue)> = {
                                let mut result = Vec::with_capacity(parsed_pairs.len());
                                for pair in parsed_pairs {
                                    let journal = pair.1;
                                    match journal.updates.last() {
                                        None => return Err(VkvError::EmptyUpdatesInJournal.into()),
                                        Some(update) => {
                                            let value = BoxedStoreValue::from(update.value.clone());
                                            result.push((pair.0, value))
                                        }
                                    }
                                }
                                result
                            };

                            return Ok(Answer::DataAccess(DataAnswer::Read(
                                DataReadAnswer::GetManyOk(GetManyOkAnswer { data }),
                            )));
                        }
                    }
                }
                DataReadInstruction::GetOne(get_one) => {
                    let target_height = match get_one.height {
                        None => self.get_height(),
                        Some(height) => height,
                    };
                    let result = self.get_at_height(&get_one.key, target_height)?;
                    return Ok(Answer::DataAccess(DataAnswer::Read(
                        DataReadAnswer::GetOneOk(GetOneOkAnswer { value: result }),
                    )));
                }
                DataReadInstruction::GetJournal(get_journal) => {
                    match self.get_journal(&get_journal.key) {
                        Err(error) => Err(error),
                        Ok(journal) => Ok(Answer::DataAccess(DataAnswer::Read(
                            DataReadAnswer::GetJournalOk(GetJournalOkAnswer { journal }),
                        ))),
                    }
                }
            },
            Instruction::Data(DataInstruction::Write(write_instruction)) => match write_instruction
            {
                DataWriteInstruction::SetMany(set_many) => {
                    let height = self.increment_chain_height()?;
                    let instruction_record = InstructionRecord {
                        api_version: IMMUXDB_VERSION,
                        instruction: instruction.clone(),
                        invalidated: false,
                    };
                    if let Err(_) =
                        self.save_instruction_record_by_height(height, &instruction_record)
                    {
                        return Err(VkvError::SaveInstructionRecordFail.into());
                    }
                    let mut count: usize = 0;
                    for target in set_many.targets.iter() {
                        match self.execute_versioned_set(&target.key, &target.value, height) {
                            Err(error) => return Err(error),
                            Ok(_result) => {
                                count += 1;
                            }
                        }
                    }
                    return Ok(Answer::DataAccess(DataAnswer::Write(
                        DataWriteAnswer::SetOk(SetOkAnswer { count }),
                    )));
                }
                DataWriteInstruction::RevertMany(revert) => {
                    let height = self.increment_chain_height()?;
                    let instruction_record = InstructionRecord {
                        api_version: IMMUXDB_VERSION,
                        instruction: instruction.clone(),
                        invalidated: false,
                    };
                    if let Err(_) =
                        self.save_instruction_record_by_height(height, &instruction_record)
                    {
                        return Err(VkvError::SaveInstructionRecordFail.into());
                    }
                    for target in revert.targets.iter() {
                        match self.revert_one(&target.key, target.height, height) {
                            Err(error) => return Err(error),
                            Ok(_result) => {}
                        }
                    }
                    return Ok(Answer::DataAccess(DataAnswer::Write(
                        DataWriteAnswer::RevertOk(RevertOkAnswer {}),
                    )));
                }
                DataWriteInstruction::RevertAll(revert_all) => {
                    let height = self.increment_chain_height()?;
                    let target_height = revert_all.target_height;
                    let instruction_record = InstructionRecord {
                        api_version: IMMUXDB_VERSION,
                        instruction: instruction.clone(),
                        invalidated: false,
                    };
                    self.save_instruction_record_by_height(height, &instruction_record)?;

                    // Find affected keys
                    let affected_keys = extract_affected_keys(&self, target_height, height)?;

                    for key in &affected_keys {
                        self.revert_one(key, target_height, height)?;
                    }

                    // Save affected for later use
                    let instruction_meta = InstructionMeta::RevertAll(RevertAllInstructionMeta {
                        deleted: false,
                        affected_keys: affected_keys.clone(),
                    });
                    self.save_instruction_meta_by_height(height, &instruction_meta)?;

                    return Ok(Answer::DataAccess(DataAnswer::Write(
                        DataWriteAnswer::RevertAllOk(RevertAllOkAnswer {
                            reverted_keys: affected_keys,
                        }),
                    )));
                }
            },
            // Transactional instructions should have been handled in TKV above VKV
            Instruction::TransactionalData(_) => {
                return Err(ImmuxError::VKV(VkvError::UnexpectedInstruction))
            }
            Instruction::TransactionMeta(_) => {
                return Err(ImmuxError::VKV(VkvError::UnexpectedInstruction))
            }
        }
    }
}
