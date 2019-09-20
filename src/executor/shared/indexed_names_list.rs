use bincode::{deserialize, serialize};

use crate::config::KVKeySigil;
use crate::declarations::basics::property_names::PropertyNameList;
use crate::declarations::basics::{GroupingLabel, PropertyNameListError, StoreKey, StoreValue};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::CoreStore;
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction, DataWriteInstruction,
    GetOneInstruction, Instruction, SetManyInstruction, SetTargetSpec,
};
use crate::storage::vkv::VkvError;

fn get_indexed_names_list_store_key(grouping: &GroupingLabel) -> StoreKey {
    let mut key_bytes = Vec::new();
    key_bytes.push(KVKeySigil::GroupingIndexedNames as u8);
    key_bytes.extend(grouping.marshal());
    StoreKey::from(key_bytes)
}

pub fn get_indexed_names_list(
    grouping: &GroupingLabel,
    core: &mut impl CoreStore,
) -> ImmuxResult<Option<PropertyNameList>> {
    let key = get_indexed_names_list_store_key(grouping);
    let instruction = Instruction::DataAccess(DataInstruction::Read(DataReadInstruction::GetOne(
        GetOneInstruction { key, height: None },
    )));
    return match core.execute(&instruction) {
        Err(ImmuxError::VKV(VkvError::MissingJournal(_))) => Ok(None),
        Err(error) => Err(error),
        Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
            match answer.value.inner() {
                None => Ok(None),
                Some(data) => match deserialize::<PropertyNameList>(data) {
                    Err(_error) => Err(PropertyNameListError::CannotParse.into()),
                    Ok(list) => Ok(Some(list)),
                },
            }
        }
        Ok(answer) => Err(ExecutorError::UnexpectedAnswerType(answer).into()),
    };
}

pub fn get_indexed_names_list_with_empty_fallback(
    grouping: &GroupingLabel,
    core: &mut impl CoreStore,
) -> ImmuxResult<PropertyNameList> {
    get_indexed_names_list(grouping, core)
        .map(|maybe_list| maybe_list.unwrap_or(PropertyNameList::new(vec![])))
}

pub fn set_indexed_names_list(
    grouping: &GroupingLabel,
    indexed_names_list: &PropertyNameList,
    core: &mut impl CoreStore,
) -> ImmuxResult<()> {
    let key = get_indexed_names_list_store_key(grouping);
    match serialize(indexed_names_list) {
        Err(_error) => return Err(ExecutorError::CannotSerialize.into()),
        Ok(data) => {
            let instruction = Instruction::DataAccess(DataInstruction::Write(
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

#[cfg(test)]
mod indexed_names_list_tests {
    use immuxdb_dev_utils::reset_db_dir;

    use crate::config::DEFAULT_PERMANENCE_PATH;
    use crate::declarations::basics::{GroupingLabel, PropertyName, PropertyNameList};
    use crate::executor::shared::indexed_names_list::get_indexed_names_list_store_key;
    use crate::executor::shared::{
        get_indexed_names_list, get_indexed_names_list_with_empty_fallback, set_indexed_names_list,
    };
    use crate::storage::core::ImmuxDBCore;
    use crate::storage::instructions::StoreNamespace;
    use crate::storage::kv::KeyValueEngine;

    fn reset_core(label: &str) -> ImmuxDBCore {
        let data_path = DEFAULT_PERMANENCE_PATH;
        reset_db_dir(&format!("{}{}", data_path, label)).unwrap();
        ImmuxDBCore::new(
            &KeyValueEngine::Rocks,
            data_path,
            &StoreNamespace::new(label.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn test_simple_get_set() {
        let mut core = reset_core("test_indexed_names_list_simple_get_set");
        let data: Vec<(GroupingLabel, PropertyNameList)> = [
            ("1", vec!["a", "b", "c"]),
            ("2", vec!["a", "b"]),
            ("empty list", vec![]),
            (
                "many items",
                vec!["a", "b", "c"]
                    .into_iter()
                    .cycle()
                    .take(100_000)
                    .collect::<Vec<&str>>(),
            ),
        ]
        .iter()
        .map(|(label, list)| {
            let grouping_label = GroupingLabel::from(*label);
            let name_list =
                PropertyNameList::new(list.iter().map(|s| PropertyName::from(*s)).collect());
            (grouping_label, name_list)
        })
        .collect();

        assert_eq!(data.len(), 4);

        for (label, list) in data.iter() {
            set_indexed_names_list(label, list, &mut core).unwrap();
        }

        for (label, list) in data.iter() {
            let list_out = get_indexed_names_list(label, &mut core).unwrap();
            assert_eq!(list.as_slice(), list_out.unwrap().as_slice());
        }

        let list_nothing = get_indexed_names_list(&GroupingLabel::from("none"), &mut core).unwrap();
        assert!(list_nothing.is_none())
    }

    #[test]
    fn test_get_with_fallback() {
        let mut core = reset_core("test_indexed_names_list_get_with_fallback");

        set_indexed_names_list(
            &GroupingLabel::from("existing"),
            &PropertyNameList::new(vec![PropertyName::from("1")]),
            &mut core,
        )
        .unwrap();

        let existing_list =
            get_indexed_names_list_with_empty_fallback(&GroupingLabel::from("existing"), &mut core)
                .unwrap();
        assert_eq!(
            existing_list.as_slice(),
            PropertyNameList::new(vec![PropertyName::from("1")]).as_slice()
        );

        let nonexisting_list =
            get_indexed_names_list_with_empty_fallback(&GroupingLabel::from("NO"), &mut core)
                .unwrap();
        assert_eq!(
            nonexisting_list.as_slice(),
            PropertyNameList::new(vec![]).as_slice()
        );
    }

    #[test]
    fn test_get_indexed_names_list_store_key() {
        let label = GroupingLabel::new(&[0x01, 0x02, 0x03]);
        let key = get_indexed_names_list_store_key(&label);
        let expected = [
            0x21, // sigil
            0x03, // length
            0x01, 0x02, 0x03, // data
        ];
        assert_eq!(key.as_slice(), expected)
    }
}
