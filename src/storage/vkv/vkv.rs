/*
 *  Versioned key-value store
**/

use std::cmp::{min, Ordering};
use std::convert::TryFrom;

use bincode::{deserialize, serialize, Error as BincodeError};

use crate::config::{KVKeySigil, MAX_RECURSION};

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
use crate::storage::vkv::height_list::HeightList;
use crate::storage::vkv::journal::UnitJournal;
use crate::storage::vkv::InstructionRecord;

#[derive(Debug)]
pub enum VkvError {
    CannotSerializeJournal,
    CannotSerializeInstructionMeta(BincodeError),
    CannotSerializeInstructionRecord,
    UnexpectedInstruction,
    DeserializationFail,
    GetInstructionRecordFail,
    GetInstructionMetaFail,
    GetHeightFail,
    CannotFindSuitableVersion,
    SaveInstructionFail,
    UpdateRecordParsing,
    JournalParsing,
    MissingJournal(StoreKey),
    TryingToRevertToFuture,
    TooManyRecursionInFindingValue,
}

fn prefix_extractor(key: &[u8]) -> &[u8] {
    match key.get(0) {
        None => return &[],
        Some(first_byte) => match KVKeySigil::try_from(*first_byte) {
            Err(_) => return &key[0..1],
            Ok(sigil) => match sigil {
                KVKeySigil::UnitJournal => {
                    let grouping_name_length = key[1];
                    let prefix_length = 1 + 1 + (grouping_name_length as usize);
                    let end = min(prefix_length, key.len());
                    return &key[0..end];
                }
                _ => return &key[0..1],
            },
        },
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

fn get_instruction_kvkey(height: &ChainHeight) -> KVKey {
    let mut result = Vec::new();
    result.push(KVKeySigil::HeightToInstructionRecord as u8);
    result.extend(height.marshal());
    result.into()
}

fn get_fallback_height() -> ChainHeight {
    ChainHeight::new(0)
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
            Err(_error) => get_fallback_height(),
            Ok(None) => get_fallback_height(),
            Ok(Some(value)) => match ChainHeight::parse(value.as_bytes()) {
                Err(_error) => get_fallback_height(),
                Ok((height, _)) => height,
            },
        }
    }

    fn set_height(&mut self, height: ChainHeight) -> ImmuxResult<()> {
        let key = &get_chain_height_kvkey();
        let value = KVValue::from(height.marshal());
        self.kv_engine.set(key, &value)
    }

    fn save_instruction_record(
        &mut self,
        height: &ChainHeight,
        record: &InstructionRecord,
    ) -> ImmuxResult<()> {
        match serialize(record) {
            Err(_error) => Err(VkvError::CannotSerializeInstructionRecord.into()),
            Ok(serialized) => {
                let key = get_instruction_kvkey(height);
                let value = KVValue::new(&serialized);
                self.kv_engine.set(&key, &value)?;
                Ok(())
            }
        }
    }

    fn load_instruction_record(&self, height: &ChainHeight) -> ImmuxResult<InstructionRecord> {
        let key = get_instruction_kvkey(height);
        match self.kv_engine.get(&key) {
            Err(_error) => Err(VkvError::GetInstructionRecordFail.into()),
            Ok(None) => Err(VkvError::GetInstructionRecordFail.into()),
            Ok(Some(value)) => match deserialize::<InstructionRecord>(value.as_bytes()) {
                Err(_error) => Err(VkvError::DeserializationFail.into()),
                Ok(instruction_record) => Ok(instruction_record),
            },
        }
    }

    fn get_journal(&self, key: &StoreKey) -> ImmuxResult<UnitJournal> {
        let kvkey = get_journal_kvkey(key);
        match self.kv_engine.get(&kvkey) {
            Err(error) => Err(error),
            Ok(None) => Err(VkvError::MissingJournal(key.to_owned()).into()),
            Ok(Some(value)) => match UnitJournal::parse(value.as_bytes()) {
                Err(_) => Err(VkvError::CannotSerializeJournal.into()),
                Ok(journal) => Ok(journal),
            },
        }
    }

    fn set_journal(&mut self, key: &StoreKey, journal: &UnitJournal) -> ImmuxResult<()> {
        let kvkey = get_journal_kvkey(key);
        let value = KVValue::new(&journal.marshal());
        self.kv_engine.set(&kvkey, &value)
    }

    fn execute_versioned_set(
        &mut self,
        store_key: &StoreKey,
        value: &StoreValue,
        height: ChainHeight,
    ) -> ImmuxResult<()> {
        let journal: UnitJournal = match self.get_journal(store_key) {
            Err(_error) => UnitJournal {
                value: value.to_owned(),
                update_heights: HeightList::new(&[height]),
            },
            Ok(mut existing_journal) => {
                existing_journal.update_heights.push(height);
                existing_journal.value = value.to_owned();
                existing_journal
            }
        };
        self.set_journal(store_key, &journal)
    }

    fn get_latest_value(&mut self, key: &StoreKey) -> ImmuxResult<StoreValue> {
        self.get_journal(key).map(|journal| journal.value)
    }

    /// Created to to prevent infinite loops
    fn get_value_after_height_recurse(
        &self,
        key: &StoreKey,
        requested_height: &ChainHeight,
        recurse_time: u16,
    ) -> ImmuxResult<StoreValue> {
        if recurse_time > MAX_RECURSION {
            return Err(VkvError::TooManyRecursionInFindingValue.into());
        }
        match self.get_journal(key) {
            Err(error) => return Err(error),
            Ok(journal) => {
                let possible_heights: Vec<_> = journal
                    .update_heights
                    .iter()
                    .take_while(|h| h <= requested_height)
                    .collect();
                for height in possible_heights.into_iter().rev() {
                    let record = self.load_instruction_record(&height)?;
                    let instruction = &record.instruction;
                    match instruction {
                        Instruction::DataAccess(DataInstruction::Write(write)) => match write {
                            DataWriteInstruction::SetMany(set_many) => {
                                for target in &set_many.targets {
                                    if target.key == *key {
                                        return Ok(target.value.clone());
                                    }
                                }
                            }
                            DataWriteInstruction::RevertMany(revert_many) => {
                                for target in &revert_many.targets {
                                    if target.key == *key {
                                        return Ok(self.get_value_after_height_recurse(
                                            key,
                                            &target.height,
                                            recurse_time + 1,
                                        )?);
                                    }
                                }
                                return Err(VkvError::CannotFindSuitableVersion.into());
                            }
                            DataWriteInstruction::RevertAll(revert_all) => {
                                return Ok(self.get_value_after_height_recurse(
                                    key,
                                    &revert_all.target_height,
                                    recurse_time + 1,
                                )?);
                            }
                        },
                        _ => return Err(VkvError::UnexpectedInstruction.into()),
                    }
                }
                return Err(VkvError::CannotFindSuitableVersion.into());
            }
        }
    }

    fn get_value_after_height(
        &self,
        key: &StoreKey,
        requested_height: &ChainHeight,
    ) -> ImmuxResult<StoreValue> {
        self.get_value_after_height_recurse(key, requested_height, 0)
    }

    fn revert_one(
        &mut self,
        key: &StoreKey,
        target_height: ChainHeight,
        next_height: ChainHeight,
    ) -> ImmuxResult<()> {
        fn find_appropriate_height(
            heights: &HeightList,
            requested_height: &ChainHeight,
        ) -> Option<ChainHeight> {
            let mut last_valid_height = None;
            for height in heights.iter() {
                if &height > requested_height {
                    break;
                } else if &height == requested_height {
                    last_valid_height = Some(height);
                    break;
                }
            }
            last_valid_height
        }

        if target_height >= next_height {
            return Err(VkvError::TryingToRevertToFuture.into());
        }
        match self.get_journal(key) {
            Err(error) => Err(error),
            Ok(mut journal) => {
                match find_appropriate_height(&journal.update_heights, &target_height) {
                    None => Err(VkvError::CannotFindSuitableVersion.into()),
                    Some(height) => {
                        let value = self.get_value_after_height(key, &height)?;
                        journal.update_heights.push(next_height);
                        journal.value = value;
                        self.set_journal(key, &journal)
                    }
                }
            }
        }
    }

    fn increment_chain_height(&mut self) -> Result<ChainHeight, ImmuxError> {
        let mut height = self.get_height();
        height.increment();
        match self.set_height(height) {
            Err(error) => return Err(error),
            Ok(_) => {}
        }
        return Ok(height);
    }
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
        if let Ok(record) = store.load_instruction_record(&height) {
            let instruction = record.instruction;
            match instruction {
                Instruction::DBSystem(_) => (),
                Instruction::TransactionMeta(_) => (),
                Instruction::DataAccess(DataInstruction::Read(_)) => (),
                Instruction::DataAccess(DataInstruction::Write(write_instruction)) => {
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
                            if let Some(keys) = record.affected_keys {
                                affected_keys.extend(keys)
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

            Instruction::DataAccess(DataInstruction::Read(read_instruction)) => {
                match read_instruction {
                    DataReadInstruction::GetMany(get_many) => {
                        match &get_many.targets {
                            GetManyTargetSpec::Keys(keys) => {
                                let mut data: Vec<(BoxedStoreKey, BoxedStoreValue)> =
                                    Vec::with_capacity(keys.len());
                                for key in keys {
                                    let value = match get_many.height {
                                        None => self.get_latest_value(&key)?,
                                        Some(height) => {
                                            self.get_value_after_height(key, &height)?
                                        }
                                    };
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
                                        match journal.value.inner() {
                                            None => {}
                                            Some(data) => {
                                                let value =
                                                    BoxedStoreValue::new(Some(data.clone()));
                                                result.push((pair.0, value));
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
                        let result = match get_one.height {
                            None => self.get_latest_value(&get_one.key)?,
                            Some(height) => self.get_value_after_height(&get_one.key, &height)?,
                        };
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
                }
            }
            Instruction::DataAccess(DataInstruction::Write(write_instruction)) => {
                // Only data writes triggers height increment and instruction record saving
                let next_height = self.increment_chain_height()?;
                match write_instruction {
                    DataWriteInstruction::SetMany(set_many) => {
                        for target in set_many.targets.iter() {
                            self.execute_versioned_set(&target.key, &target.value, next_height)?
                        }
                        let record: InstructionRecord = instruction.to_owned().into();
                        if let Err(_) = self.save_instruction_record(&next_height, &record) {
                            return Err(VkvError::SaveInstructionFail.into());
                        }
                        let count = set_many.targets.len();
                        return Ok(Answer::DataAccess(DataAnswer::Write(
                            DataWriteAnswer::SetOk(SetOkAnswer { count }),
                        )));
                    }
                    DataWriteInstruction::RevertMany(revert) => {
                        for target in revert.targets.iter() {
                            self.revert_one(&target.key, target.height, next_height)?;
                        }
                        let record: InstructionRecord = instruction.to_owned().into();
                        if let Err(_) = self.save_instruction_record(&next_height, &record) {
                            return Err(VkvError::SaveInstructionFail.into());
                        }
                        return Ok(Answer::DataAccess(DataAnswer::Write(
                            DataWriteAnswer::RevertOk(RevertOkAnswer {}),
                        )));
                    }
                    DataWriteInstruction::RevertAll(revert_all) => {
                        let target_height = revert_all.target_height;
                        if target_height >= next_height {
                            return Err(VkvError::TryingToRevertToFuture.into());
                        }

                        // Find affected keys
                        let affected_keys =
                            extract_affected_keys(&self, target_height, next_height)?;

                        for key in &affected_keys {
                            self.revert_one(key, target_height, next_height)?;
                        }

                        let record: InstructionRecord = {
                            let mut result: InstructionRecord = instruction.to_owned().into();
                            result.affected_keys = Some(affected_keys.clone());
                            result
                        };
                        if let Err(_) = self.save_instruction_record(&next_height, &record) {
                            return Err(VkvError::SaveInstructionFail.into());
                        }

                        return Ok(Answer::DataAccess(DataAnswer::Write(
                            DataWriteAnswer::RevertAllOk(RevertAllOkAnswer {
                                reverted_keys: affected_keys,
                            }),
                        )));
                    }
                }
            }
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

#[cfg(test)]
mod vkv_helper_tests {
    use super::{
        get_chain_height_kvkey, get_fallback_height, get_instruction_kvkey, get_journal_kvkey,
        prefix_extractor,
    };
    use crate::config::KVKeySigil;
    use crate::config::KVKeySigil::{
        ChainInfo, GroupingIndexedNames, HeightToInstructionRecord, ReverseIndexIdList, UnitJournal,
    };
    use crate::declarations::basics::StoreKey;
    use crate::storage::kv::KVKey;
    use crate::storage::vkv::ChainHeight;

    #[test]
    fn test_get_fallback_height() {
        assert_eq!(get_fallback_height(), ChainHeight::new(0))
    }

    #[test]
    fn test_get_chain_height_kvkey() {
        assert_eq!(
            get_chain_height_kvkey(),
            KVKey::from(vec![KVKeySigil::ChainHeight as u8])
        )
    }

    #[test]
    fn test_get_instruction_kvkey() {
        let height = ChainHeight::new(0x1314);
        let key = get_instruction_kvkey(&height);
        let expected = [HeightToInstructionRecord as u8, 0xfd, 0x14, 0x13];
        assert_eq!(key.as_bytes(), expected)
    }

    #[test]
    fn test_get_journal_key() {
        let store_key = StoreKey::new(&[0x01, 0x00, 0xff]);
        let journal_kvkey = get_journal_kvkey(&store_key);
        assert_eq!(
            journal_kvkey.as_bytes(),
            [UnitJournal as u8, 0x01, 0x00, 0xff]
        )
    }

    #[test]
    fn test_prefix_extractor() {
        let fixture: Vec<(Vec<u8>, Vec<u8>)> = vec![
            // empty key should have empty prefix
            (vec![], vec![]),
            // simple sigils
            (vec![ChainInfo as u8, 0x00, 0x00], vec![ChainInfo as u8]),
            (
                vec![HeightToInstructionRecord as u8, 0x00, 0x00],
                vec![HeightToInstructionRecord as u8],
            ),
            (
                vec![ReverseIndexIdList as u8, 0x00, 0x00],
                vec![ReverseIndexIdList as u8],
            ),
            (
                vec![GroupingIndexedNames as u8, 0x00, 0x00],
                vec![GroupingIndexedNames as u8],
            ),
            // normal unit journal
            (
                vec![UnitJournal as u8, 0x03, 0x00, 0x01, 0x02, 0xff, 0xff, 0xff],
                vec![UnitJournal as u8, 0x03, 0x00, 0x01, 0x02],
            ),
            // malformed unit journal key with missing groupLabel bytes (0xff should be 0x02)
            (
                vec![UnitJournal as u8, 0xff, 0x00, 0x01],
                vec![UnitJournal as u8, 0xff, 0x00, 0x01],
            ),
        ];
        for (key, expected_prefix) in fixture {
            let prefix = prefix_extractor(&key);
            assert_eq!(prefix.to_vec(), expected_prefix)
        }
    }
}
