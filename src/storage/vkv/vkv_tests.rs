#[cfg(test)]
mod vkv_tests {
    use immuxdb_dev_utils::reset_db_dir;

    use crate::declarations::basics::{StoreKey, StoreValue};
    use crate::declarations::errors::ImmuxError;
    use crate::storage::instructions::{
        Answer, DataAnswer, DataReadAnswer, GetJournalInstruction, GetOneInstruction, Instruction,
        SetManyInstruction, SetTargetSpec, StoreNamespace,
    };
    use crate::storage::kv::KeyValueEngine;
    use crate::storage::vkv::ChainHeight;
    use crate::storage::vkv::VkvError;
    use crate::storage::vkv::{ImmuxDBVersionedKeyValueStore, VersionedKeyValueStore};
    use crate::utils::u32_to_u8_array;

    fn make_vkv(ns_str: &str) -> ImmuxDBVersionedKeyValueStore {
        let ns = StoreNamespace::new(ns_str.as_bytes());
        let root = format!("/tmp/vkv_test/");
        reset_db_dir(&format!("{}{}", root, ns_str)).unwrap();
        ImmuxDBVersionedKeyValueStore::new(&KeyValueEngine::Rocks, &root, &ns).unwrap()
    }

    #[test]
    fn test_simple_get_set() {
        let mut vkv = make_vkv("test_simple_get_set");
        let data: Vec<(StoreKey, StoreValue)> = (0..100)
            .map(|u: u32| (u, u))
            .map(|(x, y)| {
                (
                    StoreKey::new(&u32_to_u8_array(x)),
                    StoreValue::new(Some(u32_to_u8_array(y).to_vec())),
                )
            })
            .collect();
        let input: Instruction = SetManyInstruction {
            targets: data
                .clone()
                .into_iter()
                .map(|(key, value)| SetTargetSpec { key, value })
                .collect(),
        }
        .into();
        vkv.execute(&input).unwrap();

        for (key, value) in data {
            let get_one: Instruction = GetOneInstruction { height: None, key }.into();
            match vkv.execute(&get_one).unwrap() {
                Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer))) => {
                    assert_eq!(answer.value, value)
                }
                answer => panic!("Unexpected answer {:?}", answer),
            }
        }
    }

    #[test]
    fn test_get_at_height() {
        let mut vkv = make_vkv("test_get_at_height");
        for i in 0..100 {
            let set: Instruction = SetManyInstruction {
                targets: vec![SetTargetSpec {
                    key: StoreKey::from("key"),
                    value: StoreValue::new(Some(vec![i])),
                }],
            }
            .into();
            vkv.execute(&set).unwrap();
        }
        for i in 0..100 {
            let get: Instruction = GetOneInstruction {
                height: Some(ChainHeight::new(1 + i)),
                key: StoreKey::from("key"),
            }
            .into();
            match vkv.execute(&get).unwrap() {
                Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer))) => {
                    assert_eq!(answer.value, StoreValue::new(Some(vec![i as u8])))
                }
                answer => panic!("Unexpected answer {:?}", answer),
            }
        }
    }

    #[test]
    fn test_get_journal() {
        let mut vkv = make_vkv("test_journal");

        let mut set = |key: &StoreKey, i: u8| {
            let set: Instruction = SetManyInstruction {
                targets: vec![SetTargetSpec {
                    key: key.to_owned(),
                    value: StoreValue::new(Some(vec![i])),
                }],
            }
            .into();
            vkv.execute(&set).unwrap();
        };

        let end = 100u8;

        // put data
        let key1 = StoreKey::from("key1");
        let key2 = StoreKey::from("key2");
        for i in 0..=end {
            set(&key1, i);
            set(&key2, i);
        }

        let mut get_journal = |key: &StoreKey| {
            let get: Instruction = GetJournalInstruction {
                key: key.to_owned(),
            }
            .into();
            match vkv.execute(&get).unwrap() {
                Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetJournalOk(answer))) => {
                    answer.journal
                }
                _ => panic!("Unexpected answer"),
            }
        };

        let journal1 = get_journal(&key1);
        assert_eq!(journal1.value, StoreValue::new(Some(vec![end])));
        for (i, height) in journal1.update_heights.iter().enumerate() {
            let expected = (i as u64) * 2 + 1;
            assert_eq!(height.as_u64(), expected);
        }

        let journal2 = get_journal(&key2);
        assert_eq!(journal2.value, StoreValue::new(Some(vec![end])));
        for (i, height) in journal1.update_heights.iter().enumerate() {
            let expected = i as u64 * 2 + 1;
            assert_eq!(height.as_u64(), expected);
        }
    }

    #[test]
    fn test_get_missing_journal() {
        let input_key = StoreKey::from("No such key");
        let mut vkv = make_vkv("test_get_missing_journal");
        let get: Instruction = GetJournalInstruction {
            key: input_key.clone(),
        }
        .into();

        match vkv.execute(&get) {
            Err(ImmuxError::VKV(VkvError::MissingJournal(key))) => assert_eq!(key, input_key),
            _ => panic!("Unexpected result"),
        }
    }
}
