use crate::errors::UnumError;
use crate::interfaces::instructions::{Answer, Instruction};
use crate::interfaces::result::UnumResult;
use crate::storage::kv::KeyValueEngine;
use crate::storage::vkv::{UnumVersionedKeyValueStore, VersionedKeyValueStore};

pub trait CoreStore {
    fn execute(&mut self, instruction: &Instruction) -> UnumResult<Answer>;
}

pub struct UnumCore {
    vkv: UnumVersionedKeyValueStore,
}

impl UnumCore {
    pub fn new(engine_choice: KeyValueEngine) -> Result<UnumCore, UnumError> {
        let vkv = UnumVersionedKeyValueStore::new(engine_choice)?;
        let core = UnumCore { vkv };
        Ok(core)
    }
}

impl CoreStore for UnumCore {
    fn execute(&mut self, instruction: &Instruction) -> UnumResult<Answer> {
        self.vkv.execute(instruction)
    }
}
