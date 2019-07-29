use std::ffi::OsString;

use rocksdb::{Error as RocksError, DB};

use crate::declarations::errors::ImmuxResult;
use crate::storage::kv::KeyValueStore;

#[derive(Debug)]
pub enum RocksEngineError {
    InitializationError(RocksError),
    NotFound,
    GetError(RocksError),
    PutError(RocksError),
}

pub struct RocksStore {
    data_root: String,
    namespace: Vec<u8>,
    db: DB,
}

fn get_data_dir(data_root: &str, namespace: &[u8]) -> OsString {
    let path = format!("{}{}", data_root, String::from_utf8_lossy(namespace));
    path.into()
}

fn get_new_db(data_root: &str, namespace: &[u8]) -> ImmuxResult<DB> {
    match DB::open_default(get_data_dir(data_root, namespace)) {
        Err(error) => Err(RocksEngineError::InitializationError(error).into()),
        Ok(db) => Ok(db),
    }
}

impl RocksStore {
    pub fn new(data_root: &str, namespace: &[u8]) -> ImmuxResult<RocksStore> {
        let db = get_new_db(data_root, namespace)?;
        let store = RocksStore {
            namespace: namespace.to_vec(),
            data_root: data_root.to_string(),
            db,
        };
        Ok(store)
    }
}

impl KeyValueStore for RocksStore {
    fn get(&self, key: &[u8]) -> ImmuxResult<Vec<u8>> {
        match self.db.get(key) {
            Ok(Some(value)) => Ok(value.to_vec()),
            Ok(None) => Err(RocksEngineError::NotFound.into()),
            Err(error) => Err(RocksEngineError::GetError(error).into()),
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> ImmuxResult<Vec<u8>> {
        match self.db.put(key, value) {
            Err(error) => Err(RocksEngineError::PutError(error).into()),
            Ok(_null) => Ok(vec![]),
        }
    }
    fn switch_namespace(&mut self, namespace: &[u8]) -> ImmuxResult<()> {
        self.namespace = namespace.to_vec();
        self.db = get_new_db(&self.data_root, namespace)?;
        return Ok(());
    }
    fn read_namespace(&self) -> &[u8] {
        &self.namespace
    }
}
