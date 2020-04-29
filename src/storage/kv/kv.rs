use serde::{Deserialize, Serialize};

use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::storage::kv::rocks::RocksEngineError;
use crate::storage::kv::{BoxedKVKey, BoxedKVValue, KVKey, KVKeySegment, KVNamespace, KVValue};

#[derive(Debug)]
pub enum KVError {
    RocksEngine(RocksEngineError),
}

impl From<KVError> for ImmuxError {
    fn from(error: KVError) -> Self {
        ImmuxError::KV(error)
    }
}

pub trait KeyValueStore {
    fn get(&self, kvkey: &KVKey) -> ImmuxResult<Option<KVValue>>;
    fn set(&mut self, kvkey: &KVKey, value: &KVValue) -> ImmuxResult<()>;
    fn atomic_batch_set(&mut self, pairs: &[(KVKey, KVValue)]) -> ImmuxResult<()>;
    fn switch_namespace(&mut self, namespace: &KVNamespace) -> ImmuxResult<()>;
    fn read_namespace(&self) -> KVNamespace;
    fn filter_prefix(&self, prefix: &KVKeySegment) -> Box<Vec<(BoxedKVKey, BoxedKVValue)>>;
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KeyValueEngine {
    HashMap,
    Rocks,
}

#[cfg(test)]
mod base_kv_tests {
    use std::collections::HashSet;
    use std::error::Error;

    use crate::config::{MAX_KVKEY_LENGTH, MAX_KVVALUE_LENGTH};

    use crate::storage::kv::{
        HashMapStore, KVKey, KVKeySegment, KVNamespace, KVValue, KeyValueStore, RocksStore,
    };
    use crate::utils::u64_to_u8_array;
    use immuxdb_dev_utils::reset_db_dir;

    // ========================================
    //            Test helpers
    // ========================================

    /// Artificial prefix scheme that could take 1, 2, or 3 initial bytes
    fn extract_prefix(key: &[u8]) -> &[u8] {
        match key.get(0) {
            None => &[],
            Some(first_byte) => {
                if *first_byte >= 0x80 && key.len() >= 3 {
                    return &key[..3];
                } else if *first_byte >= 0x40 && key.len() >= 2 {
                    return &key[..2];
                } else {
                    return &key[..1];
                }
            }
        }
    }

    fn get_hashmap_store() -> HashMapStore {
        let namespace = KVNamespace::from("kv_test");
        return HashMapStore::new(&namespace);
    }

    fn get_rocks_store(label: &str) -> RocksStore {
        let root = format!("/tmp/{}/", label);
        reset_db_dir(&root).unwrap();
        let namespace = KVNamespace::from(label);
        return RocksStore::new(&root, &namespace, extract_prefix).unwrap();
    }

    /// cycle_to_length(&[0,1,2], 3) -> vec![0,1,2,0,1,2,0,1,2]
    fn cycle_to_length(seed: &[u8], length: usize) -> Vec<u8> {
        seed.iter().cycle().take(length).map(|x| *x).collect()
    }

    // ========================================
    // General tests for both HashMap and Rocks
    // ========================================

    /// Setting and getting a key-value pair of maximum lengths
    fn test_get_set_max_size(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        let key_bytes = cycle_to_length(&[0, 1, 2], MAX_KVKEY_LENGTH);
        let value_bytes = cycle_to_length(&[0xff, 0xaa, 0x00], MAX_KVVALUE_LENGTH);
        let key = KVKey::from(key_bytes);
        let value = KVValue::from(value_bytes);
        store.set(&key, &value)?;
        let value_retrieved = store.get(&key)?;
        assert_eq!(Some(value), value_retrieved);
        Ok(())
    }

    /// Insert many small values and get them back
    fn test_get_set_large_quantity(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        let range = 0..1_000_000;

        fn get_key(i: u64) -> KVKey {
            KVKey::from(u64_to_u8_array(i).to_vec())
        }

        fn get_value(i: u64) -> KVValue {
            KVValue::from(u64_to_u8_array(i * 103).to_vec())
        }

        // set
        for i in range.clone() {
            let key = get_key(i);
            let value = get_value(i);
            store.set(&key, &value)?;
        }

        // get and verify
        for i in range.clone() {
            let key = get_key(i);
            let value = get_value(i);
            let value_retrieved = store.get(&key)?;
            assert_eq!(Some(value), value_retrieved);
        }

        Ok(())
    }

    /// Test writing and reading to non-sequential keys (i.e. instead of 1,2,3, it might be 1,5,2,3)
    fn test_nonsequential_access(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        let key_1 = KVKey::from("1");
        let key_2 = KVKey::from("2");
        let key_3 = KVKey::from("3");

        let value_a = KVValue::from("a");
        let value_b = KVValue::from("b");
        let value_c = KVValue::from("c");
        let value_d = KVValue::from("d");

        let ops_table: Vec<(KVKey, KVValue)> = vec![
            (key_2.clone(), value_a.clone()),
            (key_1.clone(), value_a.clone()),
            (key_3.clone(), value_a.clone()),
            (key_1.clone(), value_b.clone()),
            (key_3.clone(), value_c.clone()),
            (key_1.clone(), value_d.clone()),
        ];

        for op in &ops_table {
            store.set(&op.0, &op.1)?;
        }

        let value_1 = store.get(&key_1)?;
        let value_2 = store.get(&key_2)?;
        let value_3 = store.get(&key_3)?;

        assert_eq!(value_1, Some(value_d));
        assert_eq!(value_2, Some(value_a));
        assert_eq!(value_3, Some(value_c));

        Ok(())
    }

    /// Save data of skipping sequential keys into the store, get the data back and compare input
    /// and output.
    fn test_prefix_filter(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        let input_data: Vec<(KVKey, KVValue)> = (0..10_000)
            .map(|i| {
                // Scatter keys in the prefix space
                let key = KVKey::from(u64_to_u8_array(i * 101).to_vec());
                // Make value different from key
                let value = KVValue::from(u64_to_u8_array(i * 1021).to_vec());
                (key, value)
            })
            .collect();
        assert_eq!(input_data.len(), 10_000);

        store.atomic_batch_set(&input_data)?;

        let unique_prefixes: HashSet<KVKeySegment> = {
            let mut prefix_set = HashSet::new();
            for pair in &input_data {
                let prefix_bytes = extract_prefix(pair.0.as_bytes());
                let prefix: KVKeySegment = KVKey::new(prefix_bytes).into();
                prefix_set.insert(prefix);
            }
            prefix_set
        };

        for prefix in unique_prefixes {
            let data_from_store = store.filter_prefix(&prefix);
            let data_from_memory: Vec<_> = input_data
                .iter()
                .filter(|row| extract_prefix(row.0.as_bytes()) == prefix.as_bytes())
                .collect();

            // Check data is returned
            assert!(data_from_store.len() >= 1);

            // Check that each item in memory is provided by the store
            assert_eq!(data_from_store.len(), data_from_memory.len());
            for item in data_from_memory {
                let (key_from_memory, value_from_memory) = item;
                let pair_from_store = data_from_store
                    .iter()
                    .find(|row| row.0.as_bytes() == key_from_memory.as_bytes())
                    .unwrap();
                let (_, value_from_store) = pair_from_store;
                assert_eq!(value_from_store.as_bytes(), value_from_memory.as_bytes());
            }
        }
        Ok(())
    }

    fn test_set_many(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        let data_tables: Vec<Vec<(KVKey, KVValue)>> = vec![
            vec![("a", 1), ("b", 2), ("c", 3)], // base data
            vec![("c", 0)],                     // single datum in a set_many command
            vec![],                             // empty inserts
            vec![("b", 5), ("a", 20)],          // data existing data
            vec![("a", 10), ("d", 100)],        // update while inserting new value
        ]
            .iter()
            .map(|data_table| {
                data_table
                    .iter()
                    .map(|pair| {
                        let key = KVKey::new(pair.0.as_bytes());
                        let value = KVValue::new(&u64_to_u8_array(pair.1));
                        (key, value)
                    })
                    .collect()
            })
            .collect();

        for table in &data_tables {
            store.atomic_batch_set(table)?;
        }

        let key_a = KVKey::new("a".as_bytes());
        let key_b = KVKey::new("b".as_bytes());
        let key_c = KVKey::new("c".as_bytes());
        let key_d = KVKey::new("d".as_bytes());
        let value_a = store.get(&key_a)?;
        let value_b = store.get(&key_b)?;
        let value_c = store.get(&key_c)?;
        let value_d = store.get(&key_d)?;
        let value_expected_a = Some(KVValue::new(&u64_to_u8_array(10)));
        let value_expected_b = Some(KVValue::new(&u64_to_u8_array(5)));
        let value_expected_c = Some(KVValue::new(&u64_to_u8_array(0)));
        let value_expected_d = Some(KVValue::new(&u64_to_u8_array(100)));

        assert_eq!(value_a, value_expected_a);
        assert_eq!(value_b, value_expected_b);
        assert_eq!(value_c, value_expected_c);
        assert_eq!(value_d, value_expected_d);
        Ok(())
    }

    fn test_get_nonexistent_key(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        for i in 0..=255 {
            // key = [], [1], [2,2], [3,3,3], ...
            let key_bytes: Vec<u8> = [i as u8].iter().cycle().take(i).map(|x| *x).collect();
            let key = KVKey::from(key_bytes);
            match store.get(&key) {
                Ok(None) => (),
                Ok(_) => panic!("Should not get value from nonexistent key"),
                Err(error) => panic!("Unexpected error {:?}", error),
            }
        }
        Ok(())
    }

    /// Ensure that:
    /// (1) cannot read from other namespaces
    /// (2) cannot write to other namespaces
    fn test_switch_namespace(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        let ns_1 = KVNamespace::from("test_ns1");
        let ns_2 = KVNamespace::from("test_ns2");
        assert_ne!(store.read_namespace(), ns_1);
        assert_ne!(store.read_namespace(), ns_2);

        // Save data in namespace 1
        store.switch_namespace(&ns_1)?;
        let key_1 = KVKey::from("common-key");
        let value_1 = KVValue::from("1");
        store.set(&key_1, &value_1)?;
        let key_unique_1 = KVKey::from("only-ns1");
        let value_unique_1 = KVValue::from("ns1 value");
        store.set(&key_unique_1, &value_unique_1)?;

        // Save data in namespace 2
        store.switch_namespace(&ns_2)?;
        let key_2 = KVKey::from("common-key");
        let value_2 = KVValue::from("2");
        store.set(&key_2, &value_2)?;
        let key_unique_2 = KVKey::from("only-ns2");
        let value_unique_2 = KVValue::from("value for ns2");
        store.set(&key_unique_2, &value_unique_2)?;

        // Switch back to namespace 1 and check read & write isolation
        store.switch_namespace(&ns_1)?;
        let value_out_1 = store.get(&key_1)?;
        assert_eq!(value_out_1, Some(value_1));
        if let Some(value) = store.get(&key_unique_2).unwrap() {
            panic!(
                "Should not get value from key unique to another namespace, got {:?}",
                value
            )
        }

        // Same for namespace 2
        store.switch_namespace(&ns_2)?;
        let value_out_2 = store.get(&key_2)?;
        assert_eq!(value_out_2, Some(value_2));
        if let Some(value) = store.get(&key_unique_1).unwrap() {
            panic!(
                "Should not get value from key unique to another namespace, got {:?}",
                value
            )
        }

        Ok(())
    }

    /// Ensures that KV works with empty key (i.e. [])
    fn test_empty_key(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        let key = KVKey::new(&[]);
        let value_in = KVValue::from("value");
        store.set(&key, &value_in)?;
        let value_out = store.get(&key)?.unwrap();
        assert_eq!(value_in, value_out);
        Ok(())
    }

    /// Updates a key with different values
    fn test_key_overwrite(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        let key = KVKey::new(&[1, 2, 3]);
        let values: Vec<KVValue> = ["1", "2", "3", "4"]
            .iter()
            .map(|s| KVValue::from(*s))
            .collect();
        assert_eq!(values.len(), 4);

        for value in &values {
            store.set(&key, value)?;
            let value_out = store.get(&key)?.unwrap();
            assert_eq!(value, &value_out)
        }

        Ok(())
    }

    /// Ensure the order of set_many with the same key
    fn test_set_many_identical_keys(store: &mut impl KeyValueStore) -> Result<(), Box<dyn Error>> {
        let pairs: Vec<(KVKey, KVValue)> = vec![
            (KVKey::from("key"), KVValue::from("value-1")),
            (KVKey::from("key"), KVValue::from("value-2")),
            (KVKey::from("key"), KVValue::from("value-3")),
            (KVKey::from("key"), KVValue::from("value-4")),
            (KVKey::from("key"), KVValue::from("value-final")),
        ];
        store.atomic_batch_set(&pairs)?;
        match store.get(&KVKey::from("key"))? {
            None => panic!("Cannget read back"),
            Some(value) => assert_eq!(value, KVValue::from("value-final")),
        };
        Ok(())
    }

    // ========================================
    //    Apply tests to hashmap and rocks
    // ========================================

    #[test]
    fn test_get_set_max_size_hashmap() -> Result<(), Box<dyn Error>> {
        test_get_set_max_size(&mut get_hashmap_store())
    }

    #[test]
    fn test_get_set_max_size_rocks() -> Result<(), Box<dyn Error>> {
        test_get_set_max_size(&mut get_rocks_store("test_get_set_max_size_rocks"))
    }

    #[test]
    fn test_get_set_large_quantity_hashmap() -> Result<(), Box<dyn Error>> {
        test_get_set_large_quantity(&mut get_hashmap_store())
    }

    #[test]
    fn test_get_set_large_quantity_rocks() -> Result<(), Box<dyn Error>> {
        test_get_set_large_quantity(&mut get_rocks_store("test_get_set_large_quantity_rocks"))
    }

    #[test]
    fn test_nonsequential_access_hashmap() -> Result<(), Box<dyn Error>> {
        test_nonsequential_access(&mut get_hashmap_store())
    }

    #[test]
    fn test_nonsequential_access_rocks() -> Result<(), Box<dyn Error>> {
        test_nonsequential_access(&mut get_rocks_store("test_nonsequential_access_rocks"))
    }

    #[test]
    fn test_prefix_filter_hashmap() -> Result<(), Box<dyn Error>> {
        test_prefix_filter(&mut get_hashmap_store())
    }

    #[test]
    fn test_prefix_filter_rocks() -> Result<(), Box<dyn Error>> {
        test_prefix_filter(&mut get_rocks_store("test_prefix_filter_rocks"))
    }

    #[test]
    fn test_set_many_hashmap() -> Result<(), Box<dyn Error>> {
        test_set_many(&mut get_hashmap_store())
    }

    #[test]
    fn test_set_many_rocks() -> Result<(), Box<dyn Error>> {
        test_set_many(&mut get_rocks_store("test_set_many_rocks"))
    }

    #[test]
    fn test_get_nonexistent_rocks_hashmap() -> Result<(), Box<dyn Error>> {
        test_get_nonexistent_key(&mut get_hashmap_store())
    }

    #[test]
    fn test_get_nonexistent_rocks() -> Result<(), Box<dyn Error>> {
        test_get_nonexistent_key(&mut get_rocks_store("test_get_nonexistent_rocks"))
    }

    #[test]
    fn test_read_namespace_hashmap() -> Result<(), Box<dyn Error>> {
        let input_ns = KVNamespace::from("hello");
        let store = HashMapStore::new(&input_ns);
        let actual_ns = store.read_namespace();
        assert_eq!(actual_ns, input_ns);
        Ok(())
    }

    #[test]
    fn test_read_namespace_rocks() -> Result<(), Box<dyn Error>> {
        let input_ns = KVNamespace::from("hello");
        let store = RocksStore::new("/tmp/test_namespace", &input_ns, extract_prefix).unwrap();
        let actual_ns = store.read_namespace();
        assert_eq!(actual_ns, input_ns);
        Ok(())
    }

    #[test]
    fn test_switch_namespace_hashmap() -> Result<(), Box<dyn Error>> {
        test_switch_namespace(&mut get_hashmap_store())
    }

    #[test]
    fn test_switch_namespace_rocks() -> Result<(), Box<dyn Error>> {
        test_switch_namespace(&mut get_rocks_store("test_switch_namespace_rocks"))
    }

    #[test]
    fn test_empty_key_hashmap() -> Result<(), Box<dyn Error>> {
        test_empty_key(&mut get_hashmap_store())
    }

    #[test]
    fn test_empty_key_rocks() -> Result<(), Box<dyn Error>> {
        test_empty_key(&mut get_rocks_store("test_empty_key_rocks"))
    }

    #[test]
    fn test_key_overwrite_hashmap() -> Result<(), Box<dyn Error>> {
        test_key_overwrite(&mut get_hashmap_store())
    }

    #[test]
    fn test_key_overwrite_rocks() -> Result<(), Box<dyn Error>> {
        test_key_overwrite(&mut get_rocks_store("test_key_overwrite"))
    }

    #[test]
    fn test_set_many_identical_keys_hashmap() -> Result<(), Box<dyn Error>> {
        test_set_many_identical_keys(&mut get_hashmap_store())
    }

    #[test]
    fn test_set_many_identical_keys_rocks() -> Result<(), Box<dyn Error>> {
        test_set_many_identical_keys(&mut get_rocks_store("test_set_many_identical_keys"))
    }
}
