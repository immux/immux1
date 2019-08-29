use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::storage::instructions::{Answer, Instruction, StoreNamespace};
use crate::storage::kv::KeyValueEngine;
use crate::storage::tkv::{ImmuxDBTransactionKeyValueStore, TransactionKeyValueStore};

pub trait CoreStore {
    fn execute(&mut self, instruction: &Instruction) -> ImmuxResult<Answer>;
}

pub struct ImmuxDBCore {
    tkv: ImmuxDBTransactionKeyValueStore,
}

impl ImmuxDBCore {
    pub fn new(
        engine_choice: &KeyValueEngine,
        data_root: &str,
        namespace: &StoreNamespace,
    ) -> Result<ImmuxDBCore, ImmuxError> {
        let tkv = ImmuxDBTransactionKeyValueStore::new(engine_choice, data_root, namespace)?;
        let core = ImmuxDBCore { tkv };
        Ok(core)
    }
}

impl CoreStore for ImmuxDBCore {
    fn execute(&mut self, instruction: &Instruction) -> ImmuxResult<Answer> {
        self.tkv.execute(instruction)
    }
}
