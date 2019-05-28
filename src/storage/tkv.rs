use crate::declarations::errors::{UnumError, UnumResult};
use crate::declarations::instructions::{
    AbortTransactionOkAnswer, Answer, AtomicGetInstruction, AtomicGetOneInstruction,
    AtomicRevertAllInstruction, AtomicRevertInstruction, AtomicSetInstruction,
    CommitTransactionInstruction, CommitTransactionOkAnswer, GetOkAnswer, Instruction,
    StartTransactionOkAnswer, TransactionalGetOkAnswer, TransactionalGetOneOkAnswer,
    TransactionalRevertAllOkAnswer, TransactionalRevertOkAnswer, TransactionalSetOkAnswer,
};
use crate::storage::kv::KeyValueEngine;
use crate::storage::vkv::{
    extract_affected_keys, InstructionHeight, UnumVersionedKeyValueStore, VersionedKeyValueStore,
};
use std::collections::HashSet;

#[derive(Debug)]
pub enum TransactionError {
    TransacitonInProgress,
    TransactionNotStarted,
    UnexpectedAnswer,
    AbortInstructionError,
}

pub trait TransactionKeyValueStore {
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, UnumError>;
}

pub struct UnumTransactionKeyValueStore {
    vkv: UnumVersionedKeyValueStore,
    in_transaction: bool,

    instruction_recorder: Vec<Instruction>,
    height_before_transaction: InstructionHeight,
}

impl UnumTransactionKeyValueStore {
    pub fn new(
        engine_choice: &KeyValueEngine,
        namespace: &[u8],
    ) -> Result<UnumTransactionKeyValueStore, UnumError> {
        let vkv = UnumVersionedKeyValueStore::new(engine_choice, namespace)?;
        let in_transaction = false;
        let instruction_recorder = Vec::new();
        let height_before_transaction = 0;
        let tkv = UnumTransactionKeyValueStore {
            vkv,
            in_transaction,
            instruction_recorder,
            height_before_transaction,
        };
        return Ok(tkv);
    }

    //    TODO: Need to implement auto increasing id, See issue #93
    pub fn get_transaction_id(&self) -> u64 {
        return 1;
    }

    pub fn undo_transaction(&mut self) -> UnumResult<()> {
        let mut affected_keys = HashSet::new();
        let target_height = self.height_before_transaction;
        let current_height = self.vkv.get_current_height();
        for instruction in &self.instruction_recorder {
            match instruction {
                Instruction::AtomicSet(set_instruction) => {
                    for set_target_spec in &set_instruction.targets {
                        affected_keys.insert(set_target_spec.key.clone());
                    }
                }
                Instruction::AtomicRevert(revert_instruction) => {
                    for revert_target_spec in &revert_instruction.targets {
                        affected_keys.insert(revert_target_spec.key.clone());
                    }
                }
                Instruction::AtomicRevertAll(revert_all_instruction) => {
                    let revert_all_affected_keys =
                        extract_affected_keys(&self.vkv, target_height, current_height)?;
                    for key in revert_all_affected_keys {
                        affected_keys.insert(key);
                    }
                }
                _ => {
                    return Err(UnumError::Transaction(
                        TransactionError::AbortInstructionError,
                    ));
                }
            }
        }

        for key in affected_keys {
            self.vkv
                .invalidate_update_after_height(key.as_ref(), target_height)?;
        }
        self.vkv
            .invalidate_instruction_meta_after_height(target_height)?;

        self.vkv
            .invalidate_instruction_record_after_height(target_height)?;

        return Ok(());
    }
}

impl TransactionKeyValueStore for UnumTransactionKeyValueStore {
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, UnumError> {
        match instruction {
            Instruction::StartTransaction => {
                if !self.in_transaction {
                    self.in_transaction = true;
                    let transaction_id = self.get_transaction_id();
                    self.height_before_transaction = self.vkv.get_current_height();
                    return Ok(Answer::StartTransactionOk(StartTransactionOkAnswer {
                        transaction_id,
                    }));
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::CommitTransaction(commit_transaction_instruction) => {
                if !self.in_transaction {
                    return Err(UnumError::Transaction(
                        TransactionError::TransactionNotStarted,
                    ));
                } else {
                    self.in_transaction = false;
                    let transaction_id = commit_transaction_instruction.transaction_id;
                    self.instruction_recorder = Vec::new();
                    self.height_before_transaction = self.vkv.get_current_height();
                    return Ok(Answer::CommitTransactionOk(CommitTransactionOkAnswer {
                        transaction_id,
                    }));
                }
            }
            Instruction::AbortTransaction(abort_transaction_instruction) => {
                if !self.in_transaction {
                    return Err(UnumError::Transaction(
                        TransactionError::TransactionNotStarted,
                    ));
                } else {
                    self.in_transaction = false;
                    let transaction_id = abort_transaction_instruction.transaction_id;
                    self.undo_transaction();
                    self.instruction_recorder = Vec::new();
                    self.vkv.set_height(self.height_before_transaction)?;
                    return Ok(Answer::AbortTransactionOk(AbortTransactionOkAnswer {
                        transaction_id,
                    }));
                }
            }
            Instruction::InTransactionGet(transactional_get_instruction) => {
                if self.in_transaction {
                    let targets = transactional_get_instruction.targets.clone();
                    let transaction_id = transactional_get_instruction.transaction_id;
                    let get_instruction = AtomicGetInstruction { targets };
                    match self.vkv.execute(&Instruction::AtomicGet(get_instruction))? {
                        Answer::GetOk(get_ok_answer) => {
                            let items = get_ok_answer.items;
                            let get_ok_answer = TransactionalGetOkAnswer {
                                transaction_id,
                                items,
                            };
                            return Ok(Answer::TransactionalGetOk(get_ok_answer));
                        }
                        _ => {
                            return Err(UnumError::Transaction(TransactionError::UnexpectedAnswer));
                        }
                    }
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::InTransactionGetOne(get_one) => {
                if self.in_transaction {
                    let vkv_instruction = AtomicGetOneInstruction {
                        target: get_one.target.clone(),
                    };
                    match self
                        .vkv
                        .execute(&Instruction::AtomicGetOne(vkv_instruction))?
                    {
                        Answer::GetOneOk(answer) => {
                            let transactional_answer = TransactionalGetOneOkAnswer {
                                transaction_id: get_one.transaction_id,
                                item: answer.item,
                            };
                            return Ok(Answer::TransactionalGetOneOk(transactional_answer));
                        }
                        _ => {
                            return Err(UnumError::Transaction(TransactionError::UnexpectedAnswer));
                        }
                    }
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::InTransactionSet(transactional_set_instruction) => {
                if self.in_transaction {
                    let targets = transactional_set_instruction.targets.clone();
                    let transaction_id = transactional_set_instruction.transaction_id;
                    let deleted = false;
                    let set_instruction = AtomicSetInstruction { targets };
                    match self.vkv.execute(&Instruction::AtomicSet(set_instruction))? {
                        Answer::SetOk(set_ok_answer) => {
                            let items = set_ok_answer.items;
                            let set_ok_answer = TransactionalSetOkAnswer {
                                transaction_id,
                                items,
                            };
                            return Ok(Answer::TransactionalSetOk(set_ok_answer));
                        }
                        _ => {
                            return Err(UnumError::Transaction(TransactionError::UnexpectedAnswer));
                        }
                    }
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::InTransactionRevert(transactional_revert_instruction) => {
                if self.in_transaction {
                    let targets = transactional_revert_instruction.targets.clone();
                    let transaction_id = transactional_revert_instruction.transaction_id;
                    let deleted = false;
                    let revert_instruction = AtomicRevertInstruction { targets };
                    match self
                        .vkv
                        .execute(&Instruction::AtomicRevert(revert_instruction))?
                    {
                        Answer::RevertOk(revert_ok_answer) => {
                            let items = revert_ok_answer.items;
                            let revert_ok_answer = TransactionalRevertOkAnswer {
                                transaction_id,
                                items,
                            };
                            return Ok(Answer::TransactionalRevertOk(revert_ok_answer));
                        }
                        _ => {
                            return Err(UnumError::Transaction(TransactionError::UnexpectedAnswer));
                        }
                    }
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::InTransactionRevertAll(transactional_revert_all_instruction) => {
                if self.in_transaction {
                    let target_height = transactional_revert_all_instruction.target_height.clone();
                    let transaction_id = transactional_revert_all_instruction.transaction_id;
                    let deleted = false;
                    let revert_all_instruction = AtomicRevertAllInstruction { target_height };
                    match self
                        .vkv
                        .execute(&Instruction::AtomicRevertAll(revert_all_instruction))?
                    {
                        Answer::RevertAllOk(revert_all_ok_answer) => {
                            let reverted_keys = revert_all_ok_answer.reverted_keys;
                            let revert_all_ok_answer = TransactionalRevertAllOkAnswer {
                                transaction_id,
                                reverted_keys,
                            };
                            return Ok(Answer::TransactionalRevertAllOk(revert_all_ok_answer));
                        }
                        _ => {
                            return Err(UnumError::Transaction(TransactionError::UnexpectedAnswer));
                        }
                    }
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::AtomicGet(get_instruction) => {
                if !self.in_transaction {
                    self.vkv.execute(instruction)
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::AtomicGetOne(get_one) => {
                if !self.in_transaction {
                    self.vkv.execute(instruction)
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::AtomicSet(set_instruction) => {
                if !self.in_transaction {
                    self.instruction_recorder.push(instruction.clone());
                    self.vkv.execute(instruction)
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::AtomicRevert(revert_instruction) => {
                if !self.in_transaction {
                    self.instruction_recorder.push(instruction.clone());
                    self.vkv.execute(instruction)
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::AtomicRevertAll(revert_all_instruction) => {
                if !self.in_transaction {
                    self.instruction_recorder.push(instruction.clone());
                    self.vkv.execute(instruction)
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::SwitchNamespace(switch_namespace_instruction) => {
                if !self.in_transaction {
                    self.vkv.execute(instruction)
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
            Instruction::ReadNamespace(read_namespace_instruction) => {
                if !self.in_transaction {
                    self.vkv.execute(instruction)
                } else {
                    return Err(UnumError::Transaction(
                        TransactionError::TransacitonInProgress,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tkv_tests {
    use crate::config::{compile_config, save_config, DEFAULT_CHAIN_NAME};
    use crate::declarations::errors::UnumError;
    use crate::declarations::instructions::Instruction::InTransactionRevertAll;
    use crate::declarations::instructions::{
        AbortTransactionInstruction, Answer, AtomicGetInstruction, AtomicRevertAllInstruction,
        AtomicRevertInstruction, AtomicSetInstruction, CommitTransactionInstruction, GetTargetSpec,
        InTransactionRevertAllInstruction, InTransactionRevertInstruction,
        InTransactionSetInstruction, Instruction, RevertTargetSpec, SetTargetSpec,
    };
    use crate::storage::tkv::{
        InstructionHeight, TransactionKeyValueStore, UnumTransactionKeyValueStore,
    };
    use crate::storage::vkv::VersionedKeyValueStore;

    fn init_tkv() -> UnumTransactionKeyValueStore {
        let commandline_args = vec![];
        let config = compile_config(commandline_args);
        let namespace = "test_namespace".as_bytes();
        let mut tkv = UnumTransactionKeyValueStore::new(&config.engine_choice, namespace).unwrap();
        return tkv;
    }

    fn in_transaction_set(
        key: String,
        value: String,
        tkv: &mut UnumTransactionKeyValueStore,
    ) -> Result<Answer, UnumError> {
        let key = key.as_bytes().to_vec();
        let value = value.as_bytes().to_vec();
        let set_target_spec = SetTargetSpec { key, value };
        let transactional_set_instruction = InTransactionSetInstruction {
            targets: vec![set_target_spec],
            transaction_id: 0,
        };
        let instruction = Instruction::InTransactionSet(transactional_set_instruction);
        return tkv.execute(&instruction);
    }

    fn atomic_get(
        key: String,
        tkv: &mut UnumTransactionKeyValueStore,
    ) -> Result<Answer, UnumError> {
        let key = key.as_bytes().to_vec();
        let get_target_spec = GetTargetSpec { key, height: None };
        let targets = vec![get_target_spec];
        let get_instruction = AtomicGetInstruction { targets };
        return tkv.execute(&Instruction::AtomicGet(get_instruction));
    }

    fn in_transaction_revert(
        key: String,
        height: InstructionHeight,
        tkv: &mut UnumTransactionKeyValueStore,
    ) -> Result<Answer, UnumError> {
        let revert_target_spec = RevertTargetSpec {
            key: key.as_bytes().to_vec(),
            height,
        };
        let transactional_revert_instruction = InTransactionRevertInstruction {
            targets: vec![revert_target_spec],
            transaction_id: 0,
        };
        return tkv.execute(&Instruction::InTransactionRevert(
            transactional_revert_instruction,
        ));
    }

    fn in_transaction_revert_all(
        target_height: InstructionHeight,
        transaction_id: u64,
        tkv: &mut UnumTransactionKeyValueStore,
    ) -> Result<Answer, UnumError> {
        let in_transaction_revert_all_instruction = InTransactionRevertAllInstruction {
            target_height,
            transaction_id,
        };
        return tkv.execute(&InTransactionRevertAll(
            in_transaction_revert_all_instruction,
        ));
    }

    #[test]
    fn tkv_test() {
        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            tkv.execute(&Instruction::StartTransaction);
            let res = in_transaction_set("test_key".to_string(), "test_val".to_string(), &mut tkv)
                .unwrap();
            match res {
                Answer::TransactionalSetOk(_transactional_set_ok_answer) => {}
                _ => {
                    panic!("Expect Answer::SetOk, got other fields");
                }
            }
            let commit_transaction =
                Instruction::CommitTransaction(CommitTransactionInstruction { transaction_id: 0 });
            tkv.execute(&commit_transaction);
            assert_eq!(tkv.vkv.get_current_height(), 1);
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            tkv.execute(&Instruction::StartTransaction);
            match tkv.execute(&Instruction::StartTransaction) {
                Ok(_) => {
                    panic!("Expect panic if one transaction existed already.");
                }
                Err(error) => {}
            }
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            let commit_transaction =
                Instruction::CommitTransaction(CommitTransactionInstruction { transaction_id: 0 });
            match tkv.execute(&commit_transaction) {
                Ok(_) => {
                    panic!(
                        "Expect panic if we try to commit a transaction which is not started yet."
                    );
                }
                Err(error) => {}
            }
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            let abort_transaction =
                Instruction::AbortTransaction(AbortTransactionInstruction { transaction_id: 0 });
            match tkv.execute(&abort_transaction) {
                Ok(_) => {
                    panic!(
                        "Expect panic if we try to abort a transaction which is not started yet."
                    );
                }
                Err(error) => {}
            }
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            tkv.execute(&Instruction::StartTransaction);
            in_transaction_set("test_key".to_string(), "test_val".to_string(), &mut tkv);
            in_transaction_set("test_key1".to_string(), "test_val1".to_string(), &mut tkv);
            in_transaction_set("test_key2".to_string(), "test_val2".to_string(), &mut tkv);
            let abort_transaction =
                Instruction::AbortTransaction(AbortTransactionInstruction { transaction_id: 0 });
            tkv.execute(&abort_transaction);
            assert_eq!(tkv.height_before_transaction, 0);

            match atomic_get("test_key".to_string(), &mut tkv) {
                Ok(_) => {
                    panic!("Should not get any data after execute abort.");
                }
                Err(_) => {}
            }
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);

            tkv.execute(&Instruction::StartTransaction);
            in_transaction_set("test_key".to_string(), "test_val1".to_string(), &mut tkv);
            in_transaction_set("test_key".to_string(), "test_val2".to_string(), &mut tkv);
            in_transaction_set("test_key".to_string(), "test_val3".to_string(), &mut tkv);
            in_transaction_revert("test_key".to_string(), 1, &mut tkv);
            let commit_transaction =
                Instruction::CommitTransaction(CommitTransactionInstruction { transaction_id: 0 });
            tkv.execute(&commit_transaction);

            match atomic_get("test_key".to_string(), &mut tkv).unwrap() {
                Answer::GetOk(get_ok_answer) => {
                    assert_eq!(get_ok_answer.items[0].clone(), "test_val1".as_bytes());
                }
                _ => {}
            }
            assert_eq!(tkv.vkv.get_current_height(), 4);
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            tkv.execute(&Instruction::StartTransaction);
            in_transaction_set("test_key".to_string(), "test_val1".to_string(), &mut tkv);
            in_transaction_set("test_key".to_string(), "test_val2".to_string(), &mut tkv);
            in_transaction_set("test_key".to_string(), "test_val3".to_string(), &mut tkv);
            in_transaction_revert_all(2, 0, &mut tkv);

            let commit_transaction =
                Instruction::CommitTransaction(CommitTransactionInstruction { transaction_id: 0 });
            tkv.execute(&commit_transaction);

            match atomic_get("test_key".to_string(), &mut tkv).unwrap() {
                Answer::GetOk(get_ok_answer) => {
                    assert_eq!(get_ok_answer.items[0].clone(), "test_val2".as_bytes());
                }
                _ => {}
            }
        }
    }
}
