use crate::declarations::errors::{UnumError, UnumResult};
use crate::declarations::instructions::{Answer, Instruction};
use crate::storage::kv::KeyValueEngine;
use crate::storage::vkv::{UnumVersionedKeyValueStore, VersionedKeyValueStore};

pub trait CoreStore {
    fn execute(&mut self, instruction: &Instruction) -> UnumResult<Answer>;
}

pub struct UnumCore {
    vkv: UnumVersionedKeyValueStore,
}

impl UnumCore {
    pub fn new(engine_choice: &KeyValueEngine, namespace: &[u8]) -> Result<UnumCore, UnumError> {
        let vkv = UnumVersionedKeyValueStore::new(engine_choice, namespace)?;
        let core = UnumCore { vkv };
        Ok(core)
    }
}

impl CoreStore for UnumCore {
    fn execute(&mut self, instruction: &Instruction) -> UnumResult<Answer> {
        self.vkv.execute(instruction)
    }
}
