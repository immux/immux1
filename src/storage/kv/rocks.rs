use std::ffi::OsString;

use rocksdb::DB;

use crate::config::DEFAULT_PERMANENCE_PATH;
use crate::declarations::errors::{UnumError, UnumResult};
use crate::storage::kv::KeyValueStore;

pub struct RocksStore {
    namespace: Vec<u8>,
    db: DB,
}

fn get_data_dir(namespace: &[u8]) -> OsString {
    let path = format!(
        "{}{}",
        DEFAULT_PERMANENCE_PATH,
        String::from_utf8_lossy(namespace)
    );
    path.into()
}

fn get_new_db(namespace: &[u8]) -> UnumResult<DB> {
    match DB::open_default(get_data_dir(namespace)) {
        Err(_err) => Err(UnumError::InitializationFail),
        Ok(db) => Ok(db),
    }
}

impl RocksStore {
    pub fn new(namespace: &[u8]) -> UnumResult<RocksStore> {
        let db = get_new_db(namespace)?;
        let store = RocksStore {
            namespace: namespace.to_vec(),
            db,
        };
        Ok(store)
    }
}

impl KeyValueStore for RocksStore {
    fn get(&self, key: &[u8]) -> UnumResult<Vec<u8>> {
        match self.db.get(key) {
            Ok(Some(value)) => Ok(value.to_vec()),
            Ok(None) => Err(UnumError::ReadError),
            Err(_error) => Err(UnumError::ReadError),
        }
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> UnumResult<Vec<u8>> {
        match self.db.put(key, value) {
            Err(_error) => Err(UnumError::WriteError),
            Ok(_) => Ok(vec![]),
        }
    }
    fn switch_namespace(&mut self, namespace: &[u8]) -> UnumResult<()> {
        self.namespace = namespace.to_vec();
        self.db = get_new_db(namespace)?;
        return Ok(());
    }
    fn read_namespace(&self) -> &[u8] {
        &self.namespace
    }
}
