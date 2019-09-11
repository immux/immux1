use bincode::{deserialize, serialize};

use crate::config::KVKeySigil;
use crate::declarations::basics::property_names::PropertyNameList;
use crate::declarations::basics::{GroupingLabel, PropertyNameListError, StoreKey, StoreValue};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction, DataWriteInstruction,
    GetOneInstruction, Instruction, SetManyInstruction, SetTargetSpec,
};
use crate::storage::vkv::VkvError;

fn get_indexed_names_list_store_key(grouping: &GroupingLabel) -> StoreKey {
    let grouping_bytes: Vec<u8> = grouping.to_owned().into();
    let mut key_bytes = Vec::new();
    key_bytes.push(KVKeySigil::GroupingIndexedNames as u8);
    key_bytes.push(grouping_bytes.len() as u8);
    key_bytes.extend(grouping_bytes);
    StoreKey::from(key_bytes)
}

pub fn get_indexed_names_list(
    grouping: &GroupingLabel,
    core: &mut ImmuxDBCore,
) -> ImmuxResult<PropertyNameList> {
    let key = get_indexed_names_list_store_key(grouping);
    let instruction = Instruction::Data(DataInstruction::Read(DataReadInstruction::GetOne(
        GetOneInstruction { key, height: None },
    )));
    return match core.execute(&instruction) {
        Err(ImmuxError::VKV(VkvError::MissingJournal(_))) => {
            Err(ExecutorError::NoIndexedNamesList(grouping.clone()).into())
        }
        Err(error) => Err(error),
        Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
            match answer.value.inner() {
                None => Err(ExecutorError::NoIndexedNamesList(grouping.clone()).into()),
                Some(data) => match deserialize::<PropertyNameList>(data) {
                    Err(_error) => Err(PropertyNameListError::CannotParse.into()),
                    Ok(list) => Ok(list),
                },
            }
        }
        Ok(answer) => Err(ExecutorError::UnexpectedAnswerType(answer).into()),
    };
}

pub fn get_indexed_names_list_with_fallback(
    grouping: &GroupingLabel,
    core: &mut ImmuxDBCore,
) -> ImmuxResult<PropertyNameList> {
    match get_indexed_names_list(grouping, core) {
        Err(ImmuxError::Executor(ExecutorError::NoIndexedNamesList(_))) => {
            Ok(PropertyNameList::new(vec![]))
        }
        Err(error) => return Err(error),
        Ok(list) => Ok(list),
    }
}

pub fn set_indexed_names_list(
    grouping: &GroupingLabel,
    indexed_names_list: &PropertyNameList,
    core: &mut ImmuxDBCore,
) -> ImmuxResult<()> {
    let key = get_indexed_names_list_store_key(grouping);
    match serialize(indexed_names_list) {
        Err(_error) => return Err(ExecutorError::CannotSerialize.into()),
        Ok(data) => {
            let instruction = Instruction::Data(DataInstruction::Write(
                DataWriteInstruction::SetMany(SetManyInstruction {
                    targets: vec![SetTargetSpec {
                        key,
                        value: StoreValue::new(Some(data)),
                    }],
                }),
            ));
            match core.execute(&instruction) {
                Err(error) => Err(error),
                Ok(_) => Ok(()),
            }
        }
    }
}
