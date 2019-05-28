use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{Answer, Instruction};
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
        namespace: &[u8],
    ) -> Result<ImmuxDBCore, ImmuxError> {
        let tkv = ImmuxDBTransactionKeyValueStore::new(engine_choice, namespace)?;
        let core = ImmuxDBCore { tkv };
        Ok(core)
    }
}

impl CoreStore for ImmuxDBCore {
    fn execute(&mut self, instruction: &Instruction) -> ImmuxResult<Answer> {
        self.tkv.execute(instruction)
    }
}
