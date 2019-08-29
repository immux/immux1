use std::ffi::OsString;

use rocksdb::{
    Direction, Error as RocksError, IteratorMode, Options, ReadOptions, SliceTransform, WriteBatch,
    DB,
};

use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::storage::kv::{
    BoxedKVKey, BoxedKVValue, KVError, KVKey, KVKeySegment, KVNamespace, KVValue, KeyValueStore,
};

pub type PrefixExtractor = fn(&[u8]) -> &[u8];

#[derive(Debug)]
pub enum RocksEngineError {
    InitializationError(RocksError),
    GetError(RocksError),
    PutError(RocksError),
    BatchPutError(RocksError),
    BatchWriteError(RocksError),
}

impl From<RocksEngineError> for ImmuxError {
    fn from(error: RocksEngineError) -> ImmuxError {
        ImmuxError::KV(KVError::RocksEngine(error))
    }
}

pub struct RocksStore {
    data_root: String,
    namespace: KVNamespace,
    db: DB,
    extractor: PrefixExtractor,
}

fn get_data_dir(data_root: &str, namespace: &KVNamespace) -> OsString {
    let path = format!("{}{}", data_root, namespace.to_string());
    path.into()
}

fn get_new_db(
    data_root: &str,
    namespace: &KVNamespace,
    prefix_extractor: PrefixExtractor,
) -> ImmuxResult<DB> {
    let mut options = Options::default();
    options.create_if_missing(true);
    options.set_prefix_extractor(SliceTransform::create("all", prefix_extractor, None));
    match DB::open(&options, get_data_dir(data_root, namespace)) {
        Err(error) => Err(RocksEngineError::InitializationError(error).into()),
        Ok(db) => Ok(db),
    }
}

impl RocksStore {
    pub fn new(
        data_root: &str,
        namespace: &KVNamespace,
        prefix_extractor: PrefixExtractor,
    ) -> ImmuxResult<RocksStore> {
        let db = get_new_db(data_root, namespace, prefix_extractor)?;
        let store = RocksStore {
            namespace: namespace.to_owned(),
            data_root: data_root.to_string(),
            db,
            extractor: prefix_extractor,
        };
        Ok(store)
    }
}

impl KeyValueStore for RocksStore {
    fn get(&self, key: &KVKey) -> ImmuxResult<KVValue> {
        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => Ok(value.to_vec().into()),
            Ok(None) => Err(KVError::NotFound(key.clone()).into()),
            Err(error) => Err(RocksEngineError::GetError(error).into()),
        }
    }

    fn set(&mut self, key: &KVKey, value: &KVValue) -> ImmuxResult<()> {
        match self.db.put(key.as_bytes(), value.as_bytes()) {
            Err(error) => Err(RocksEngineError::PutError(error).into()),
            Ok(_) => Ok(()),
        }
    }

    fn set_many(&mut self, pairs: &[(KVKey, KVValue)]) -> ImmuxResult<()> {
        let mut batch = WriteBatch::default();
        for pair in pairs {
            let (key, value) = pair;
            match batch.put(key.as_bytes(), value.as_bytes()) {
                Err(error) => return Err(RocksEngineError::BatchPutError(error).into()),
                Ok(_) => {}
            };
        }
        match self.db.write(batch) {
            Err(error) => Err(RocksEngineError::BatchWriteError(error).into()),
            Ok(_) => Ok(()),
        }
    }

    fn switch_namespace(&mut self, namespace: &KVNamespace) -> ImmuxResult<()> {
        self.namespace = namespace.to_owned();
        self.db = get_new_db(&self.data_root, namespace, self.extractor)?;
        Ok(())
    }

    fn read_namespace(&self) -> KVNamespace {
        self.namespace.clone()
    }

    fn filter_prefix(&self, prefix: &KVKeySegment) -> Box<Vec<(BoxedKVKey, BoxedKVValue)>> {
        let read_options = ReadOptions::default();
        let iterator = self
            .db
            .iterator_opt(
                IteratorMode::From(prefix.as_bytes(), Direction::Forward),
                &read_options,
            )
            .take_while(|pair| pair.0.starts_with(prefix.as_bytes()));
        let data: Vec<_> = iterator
            .map(|item| (BoxedKVKey::new(item.0), BoxedKVValue::new(item.1)))
            .collect();
        Box::new(data)
    }
}

#[cfg(test)]
mod rocks_specific_tests {
    use crate::storage::kv::{KVNamespace, RocksStore};

    #[test]
    #[should_panic]
    fn test_invalid_path_error() {
        fn prefix_extract(key: &[u8]) -> &[u8] {
            return key;
        }
        let ns = KVNamespace::from("");
        RocksStore::new("\0\\", &ns, prefix_extract).unwrap();
    }
}
