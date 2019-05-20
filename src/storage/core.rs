use crate::declarations::errors::{UnumError, UnumResult};
use crate::declarations::instructions::{Answer, Instruction};
use crate::storage::kv::KeyValueEngine;
use crate::storage::tkv::{TransactionKeyValueStore, UnumTransactionKeyValueStore};

pub trait CoreStore {
    fn execute(&mut self, instruction: &Instruction) -> UnumResult<Answer>;
}

pub struct UnumCore {
    tkv: UnumTransactionKeyValueStore,
}

impl UnumCore {
    pub fn new(engine_choice: &KeyValueEngine, namespace: &[u8]) -> Result<UnumCore, UnumError> {
        let tkv = UnumTransactionKeyValueStore::new(engine_choice, namespace)?;
        let core = UnumCore { tkv };
        Ok(core)
    }
}

impl CoreStore for UnumCore {
    fn execute(&mut self, instruction: &Instruction) -> UnumResult<Answer> {
        self.tkv.execute(instruction)
    }
}
