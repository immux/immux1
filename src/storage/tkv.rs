use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{
    AbortTransactionOkAnswer, Answer, AtomicGetInstruction, AtomicGetOneInstruction,
    AtomicRevertAllInstruction, AtomicRevertInstruction, AtomicSetInstruction,
    CommitTransactionOkAnswer, Instruction, StartTransactionOkAnswer, TransactionPendingAnswer,
    TransactionalGetOkAnswer, TransactionalGetOneOkAnswer, TransactionalRevertAllOkAnswer,
    TransactionalRevertOkAnswer, TransactionalSetOkAnswer,
};
use crate::storage::kv::KeyValueEngine;
use crate::storage::vkv::{
    extract_affected_keys, ImmuxDBVersionedKeyValueStore, InstructionHeight, VersionedKeyValueStore,
};
use std::collections::{HashSet, VecDeque};

#[derive(Debug)]
pub enum TransactionError {
    TransactionInProgress,
    TransactionNotStarted,
    UnexpectedAnswer,
    AbortInstructionError,
}

pub trait TransactionKeyValueStore {
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, ImmuxError>;
}

pub struct ImmuxDBTransactionKeyValueStore {
    vkv: ImmuxDBVersionedKeyValueStore,
    executed_instructions: Vec<Instruction>,
    height_before_transaction: InstructionHeight,
    queue: VecDeque<u64>,
    current_active_transaction_id: u64,
}

impl ImmuxDBTransactionKeyValueStore {
    pub fn new(
        engine_choice: &KeyValueEngine,
        namespace: &[u8],
    ) -> Result<ImmuxDBTransactionKeyValueStore, ImmuxError> {
        let vkv = ImmuxDBVersionedKeyValueStore::new(engine_choice, namespace)?;
        let executed_instructions = Vec::new();
        let height_before_transaction = 0;
        let queue = VecDeque::new();
        let current_active_transaction_id = 1;

        let tkv = ImmuxDBTransactionKeyValueStore {
            vkv,
            executed_instructions,
            height_before_transaction,
            queue,
            current_active_transaction_id,
        };
        return Ok(tkv);
    }

    pub fn undo_transaction(&mut self) -> ImmuxResult<()> {
        let mut affected_keys = HashSet::new();
        let target_height = self.height_before_transaction;
        let current_height = self.vkv.get_current_height();
        for instruction in &self.executed_instructions {
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
                Instruction::AtomicRevertAll(_revert_all_instruction) => {
                    let revert_all_affected_keys =
                        extract_affected_keys(&self.vkv, target_height, current_height)?;
                    for key in revert_all_affected_keys {
                        affected_keys.insert(key);
                    }
                }
                _ => {
                    return Err(ImmuxError::Transaction(
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

impl TransactionKeyValueStore for ImmuxDBTransactionKeyValueStore {
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, ImmuxError> {
        match instruction {
            Instruction::StartTransaction => {
                self.current_active_transaction_id += 1;
                let transaction_id = self.current_active_transaction_id;
                if self.queue.is_empty() {
                    self.queue.push_back(transaction_id);
                    self.height_before_transaction = self.vkv.get_current_height();
                    self.executed_instructions.clear();
                    return Ok(Answer::StartTransactionOk(StartTransactionOkAnswer {
                        transaction_id,
                    }));
                } else {
                    self.queue.push_back(transaction_id);
                    let transaction_pending_answer = TransactionPendingAnswer { transaction_id };
                    return Ok(Answer::AppendTransactionOk(transaction_pending_answer));
                }
            }
            Instruction::CommitTransaction(commit_transaction_instruction) => {
                match self.queue.front() {
                    Some(transaction_id) => {
                        if *transaction_id == commit_transaction_instruction.transaction_id {
                            let transaction_id = commit_transaction_instruction.transaction_id;
                            self.executed_instructions.clear();
                            self.height_before_transaction = self.vkv.get_current_height();
                            self.queue.pop_front();
                            let next_active_transaction_id = self.queue.pop_front();
                            return Ok(Answer::CommitTransactionOk(CommitTransactionOkAnswer {
                                commited_transaction_id: transaction_id,
                                next_active_transaction_id,
                            }));
                        } else {
                            return Err(ImmuxError::Transaction(
                                TransactionError::TransactionNotStarted,
                            ));
                        }
                    }
                    None => {
                        return Err(ImmuxError::Transaction(
                            TransactionError::TransactionNotStarted,
                        ));
                    }
                }
            }
            Instruction::AbortTransaction(abort_transaction_instruction) => {
                match self.queue.front() {
                    Some(transaction_id) => {
                        if *transaction_id == abort_transaction_instruction.transaction_id {
                            let transaction_id = abort_transaction_instruction.transaction_id;
                            match self.undo_transaction() {
                                Ok(()) => {}
                                Err(error) => return Err(error),
                            };
                            self.executed_instructions.clear();
                            self.vkv.set_height(self.height_before_transaction)?;
                            self.queue.pop_front();
                            return Ok(Answer::AbortTransactionOk(AbortTransactionOkAnswer {
                                transaction_id,
                            }));
                        } else {
                            return Err(ImmuxError::Transaction(
                                TransactionError::TransactionNotStarted,
                            ));
                        }
                    }
                    None => {
                        return Err(ImmuxError::Transaction(
                            TransactionError::TransactionNotStarted,
                        ));
                    }
                }
            }
            Instruction::InTransactionGet(transactional_get_instruction) => {
                match self.queue.front() {
                    Some(transaction_id) => {
                        if *transaction_id == transactional_get_instruction.transaction_id {
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
                                    return Err(ImmuxError::Transaction(
                                        TransactionError::UnexpectedAnswer,
                                    ));
                                }
                            }
                        } else {
                            return Err(ImmuxError::Transaction(
                                TransactionError::TransactionNotStarted,
                            ));
                        }
                    }
                    None => {
                        return Err(ImmuxError::Transaction(
                            TransactionError::TransactionNotStarted,
                        ));
                    }
                }
            }
            Instruction::InTransactionGetOne(get_one) => match self.queue.front() {
                Some(transaction_id) => {
                    if *transaction_id == get_one.transaction_id {
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
                                return Err(ImmuxError::Transaction(
                                    TransactionError::UnexpectedAnswer,
                                ));
                            }
                        }
                    } else {
                        return Err(ImmuxError::Transaction(
                            TransactionError::TransactionNotStarted,
                        ));
                    }
                }
                None => {
                    return Err(ImmuxError::Transaction(
                        TransactionError::TransactionNotStarted,
                    ));
                }
            },
            Instruction::InTransactionSet(transactional_set_instruction) => {
                match self.queue.front() {
                    Some(transaction_id) => {
                        if *transaction_id == transactional_set_instruction.transaction_id {
                            let targets = transactional_set_instruction.targets.clone();
                            let transaction_id = transactional_set_instruction.transaction_id;
                            let set_instruction = AtomicSetInstruction { targets };
                            let instruction = Instruction::AtomicSet(set_instruction);
                            match self.vkv.execute(&instruction)? {
                                Answer::SetOk(set_ok_answer) => {
                                    let items = set_ok_answer.items;
                                    self.executed_instructions.push(instruction);
                                    let set_ok_answer = TransactionalSetOkAnswer {
                                        transaction_id,
                                        items,
                                    };
                                    return Ok(Answer::TransactionalSetOk(set_ok_answer));
                                }
                                _ => {
                                    return Err(ImmuxError::Transaction(
                                        TransactionError::UnexpectedAnswer,
                                    ));
                                }
                            }
                        } else {
                            return Err(ImmuxError::Transaction(
                                TransactionError::TransactionNotStarted,
                            ));
                        }
                    }
                    None => {
                        return Err(ImmuxError::Transaction(
                            TransactionError::TransactionNotStarted,
                        ));
                    }
                }
            }
            Instruction::InTransactionRevert(transactional_revert_instruction) => {
                match self.queue.front() {
                    Some(transaction_id) => {
                        if *transaction_id == transactional_revert_instruction.transaction_id {
                            let targets = transactional_revert_instruction.targets.clone();
                            let transaction_id = transactional_revert_instruction.transaction_id;
                            let revert_instruction = AtomicRevertInstruction { targets };
                            let instruction = Instruction::AtomicRevert(revert_instruction);
                            match self.vkv.execute(&instruction)? {
                                Answer::RevertOk(revert_ok_answer) => {
                                    let items = revert_ok_answer.items;
                                    self.executed_instructions.push(instruction);
                                    let revert_ok_answer = TransactionalRevertOkAnswer {
                                        transaction_id,
                                        items,
                                    };
                                    return Ok(Answer::TransactionalRevertOk(revert_ok_answer));
                                }
                                _ => {
                                    return Err(ImmuxError::Transaction(
                                        TransactionError::UnexpectedAnswer,
                                    ));
                                }
                            }
                        } else {
                            return Err(ImmuxError::Transaction(
                                TransactionError::TransactionNotStarted,
                            ));
                        }
                    }
                    None => {
                        return Err(ImmuxError::Transaction(
                            TransactionError::TransactionNotStarted,
                        ));
                    }
                }
            }
            Instruction::InTransactionRevertAll(transactional_revert_all_instruction) => match self
                .queue
                .front()
            {
                Some(transaction_id) => {
                    if *transaction_id == transactional_revert_all_instruction.transaction_id {
                        let target_height =
                            transactional_revert_all_instruction.target_height.clone();
                        let transaction_id = transactional_revert_all_instruction.transaction_id;
                        let revert_all_instruction = AtomicRevertAllInstruction { target_height };
                        let instruction = Instruction::AtomicRevertAll(revert_all_instruction);
                        match self.vkv.execute(&instruction)? {
                            Answer::RevertAllOk(revert_all_ok_answer) => {
                                let reverted_keys = revert_all_ok_answer.reverted_keys;
                                self.executed_instructions.push(instruction);
                                let revert_all_ok_answer = TransactionalRevertAllOkAnswer {
                                    transaction_id,
                                    reverted_keys,
                                };
                                return Ok(Answer::TransactionalRevertAllOk(revert_all_ok_answer));
                            }
                            _ => {
                                return Err(ImmuxError::Transaction(
                                    TransactionError::UnexpectedAnswer,
                                ));
                            }
                        }
                    } else {
                        return Err(ImmuxError::Transaction(
                            TransactionError::TransactionNotStarted,
                        ));
                    }
                }
                None => {
                    return Err(ImmuxError::Transaction(
                        TransactionError::TransactionNotStarted,
                    ));
                }
            },
            Instruction::AtomicGet(_get_instruction) => {
                if self.queue.is_empty() {
                    self.vkv.execute(instruction)
                } else {
                    return Err(ImmuxError::Transaction(
                        TransactionError::TransactionInProgress,
                    ));
                }
            }

            Instruction::AtomicGetOne(_get_one) => {
                if self.queue.is_empty() {
                    self.vkv.execute(instruction)
                } else {
                    return Err(ImmuxError::Transaction(
                        TransactionError::TransactionInProgress,
                    ));
                }
            }
            Instruction::AtomicSet(_set_instruction) => {
                if self.queue.is_empty() {
                    self.vkv.execute(instruction)
                } else {
                    return Err(ImmuxError::Transaction(
                        TransactionError::TransactionInProgress,
                    ));
                }
            }
            Instruction::AtomicRevert(_revert_instruction) => {
                if self.queue.is_empty() {
                    self.vkv.execute(instruction)
                } else {
                    return Err(ImmuxError::Transaction(
                        TransactionError::TransactionInProgress,
                    ));
                }
            }
            Instruction::AtomicRevertAll(_revert_all_instruction) => {
                if self.queue.is_empty() {
                    self.vkv.execute(instruction)
                } else {
                    return Err(ImmuxError::Transaction(
                        TransactionError::TransactionInProgress,
                    ));
                }
            }
            Instruction::SwitchNamespace(_switch_namespace_instruction) => {
                if self.queue.is_empty() {
                    self.vkv.execute(instruction)
                } else {
                    return Err(ImmuxError::Transaction(
                        TransactionError::TransactionInProgress,
                    ));
                }
            }
            Instruction::ReadNamespace(_read_namespace_instruction) => {
                if self.queue.is_empty() {
                    self.vkv.execute(instruction)
                } else {
                    return Err(ImmuxError::Transaction(
                        TransactionError::TransactionInProgress,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tkv_tests {
    use crate::config::compile_config;
    use crate::declarations::errors::ImmuxError;
    use crate::declarations::instructions::Instruction::InTransactionRevertAll;
    use crate::declarations::instructions::{
        AbortTransactionInstruction, Answer, AtomicGetInstruction, AtomicSetInstruction,
        CommitTransactionInstruction, GetTargetSpec, InTransactionGetOneInstruction,
        InTransactionRevertAllInstruction, InTransactionRevertInstruction,
        InTransactionSetInstruction, Instruction, RevertTargetSpec, SetTargetSpec,
    };
    use crate::storage::tkv::{
        ImmuxDBTransactionKeyValueStore, InstructionHeight, TransactionError,
        TransactionKeyValueStore,
    };
    use crate::storage::vkv::VersionedKeyValueStore;

    fn init_tkv() -> ImmuxDBTransactionKeyValueStore {
        let commandline_args = vec![String::from(""), String::from("--memory"), String::from("")];
        let config = compile_config(commandline_args);
        let namespace = "test_namespace".as_bytes();
        let tkv = ImmuxDBTransactionKeyValueStore::new(&config.engine_choice, namespace).unwrap();
        return tkv;
    }

    fn get_start_transaction_insturction() -> Instruction {
        return Instruction::StartTransaction;
    }

    fn start_transaction(tkv: &mut ImmuxDBTransactionKeyValueStore) -> Result<u64, ImmuxError> {
        let start_transaction_instruction = get_start_transaction_insturction();
        let res = tkv.execute(&start_transaction_instruction);
        match res {
            Ok(answer) => match answer {
                Answer::StartTransactionOk(start_transaction_ok_answer) => {
                    return Ok(start_transaction_ok_answer.transaction_id);
                }
                Answer::AppendTransactionOk(append_transaction_ok_answer) => {
                    return Ok(append_transaction_ok_answer.transaction_id);
                }
                _ => {
                    return Err(ImmuxError::Transaction(TransactionError::UnexpectedAnswer));
                }
            },
            Err(error) => {
                return Err(error);
            }
        }
    }

    fn get_transactional_set_instruction(
        key: String,
        value: String,
        transaction_id: u64,
    ) -> Instruction {
        let key = key.as_bytes().to_vec();
        let value = value.as_bytes().to_vec();
        let set_target_spec = SetTargetSpec { key, value };
        let transactional_set_instruction = InTransactionSetInstruction {
            targets: vec![set_target_spec],
            transaction_id,
        };
        return Instruction::InTransactionSet(transactional_set_instruction);
    }

    fn in_transaction_set(
        key: String,
        value: String,
        transaction_id: u64,
        tkv: &mut ImmuxDBTransactionKeyValueStore,
    ) -> Result<Answer, ImmuxError> {
        let instruction = get_transactional_set_instruction(key, value, transaction_id);
        return tkv.execute(&instruction);
    }

    fn get_transactional_get_one_instruction(key: String, transaction_id: u64) -> Instruction {
        let key = key.as_bytes().to_vec();
        let get_target_spec = GetTargetSpec { key, height: None };
        let transactional_get_one_instruction = InTransactionGetOneInstruction {
            target: get_target_spec,
            transaction_id,
        };
        return Instruction::InTransactionGetOne(transactional_get_one_instruction);
    }

    fn in_transaction_get_one_string(
        key: String,
        transaction_id: u64,
        tkv: &mut ImmuxDBTransactionKeyValueStore,
    ) -> Result<String, ImmuxError> {
        let instruction = get_transactional_get_one_instruction(key, transaction_id);
        match tkv.execute(&instruction) {
            Ok(answer) => match answer {
                Answer::TransactionalGetOneOk(get_one_ok_answer) => {
                    let string_vec = get_one_ok_answer.item;
                    let res = String::from_utf8(string_vec).unwrap();
                    return Ok(res);
                }
                _ => return Err(ImmuxError::Transaction(TransactionError::UnexpectedAnswer)),
            },
            Err(error) => {
                return Err(error);
            }
        }
    }

    fn get_atomic_set_instruction(key: String, value: String) -> Instruction {
        let key = key.as_bytes().to_vec();
        let value = value.as_bytes().to_vec();
        let set_target_spec = SetTargetSpec { key, value };
        let atomic_set_instruction = AtomicSetInstruction {
            targets: vec![set_target_spec],
        };
        return Instruction::AtomicSet(atomic_set_instruction);
    }

    fn atomic_set(
        key: String,
        value: String,
        tkv: &mut ImmuxDBTransactionKeyValueStore,
    ) -> Result<Answer, ImmuxError> {
        let instruction = get_atomic_set_instruction(key, value);
        return tkv.execute(&instruction);
    }

    fn get_atomic_get_instruction(key: String) -> Instruction {
        let key = key.as_bytes().to_vec();
        let get_target_spec = GetTargetSpec { key, height: None };
        let targets = vec![get_target_spec];
        let get_instruction = AtomicGetInstruction { targets };
        return Instruction::AtomicGet(get_instruction);
    }

    fn atomic_get_string(
        key: String,
        tkv: &mut ImmuxDBTransactionKeyValueStore,
    ) -> Result<String, ImmuxError> {
        let instruction = get_atomic_get_instruction(key);
        match tkv.execute(&instruction) {
            Ok(answer) => match answer {
                Answer::GetOk(get_ok_answer) => {
                    let string_vec = &get_ok_answer.items[0];
                    let res = String::from_utf8(string_vec.clone()).unwrap();
                    return Ok(res);
                }
                _ => return Err(ImmuxError::Transaction(TransactionError::UnexpectedAnswer)),
            },
            Err(error) => {
                return Err(error);
            }
        }
    }

    fn get_transactional_revert_instruction(
        key: String,
        height: InstructionHeight,
        transaction_id: u64,
    ) -> Instruction {
        let revert_target_spec = RevertTargetSpec {
            key: key.as_bytes().to_vec(),
            height,
        };
        let transactional_revert_instruction = InTransactionRevertInstruction {
            targets: vec![revert_target_spec],
            transaction_id,
        };
        return Instruction::InTransactionRevert(transactional_revert_instruction);
    }

    fn in_transaction_revert(
        key: String,
        height: InstructionHeight,
        tkv: &mut ImmuxDBTransactionKeyValueStore,
        transaction_id: u64,
    ) -> Result<Answer, ImmuxError> {
        let instruction = get_transactional_revert_instruction(key, height, transaction_id);
        return tkv.execute(&instruction);
    }

    fn get_transactional_revert_all_instruction(
        target_height: InstructionHeight,
        transaction_id: u64,
    ) -> Instruction {
        let in_transaction_revert_all_instruction = InTransactionRevertAllInstruction {
            target_height,
            transaction_id,
        };
        return InTransactionRevertAll(in_transaction_revert_all_instruction);
    }

    fn in_transaction_revert_all(
        target_height: InstructionHeight,
        transaction_id: u64,
        tkv: &mut ImmuxDBTransactionKeyValueStore,
    ) -> Result<Answer, ImmuxError> {
        let instruction = get_transactional_revert_all_instruction(target_height, transaction_id);
        return tkv.execute(&instruction);
    }

    #[test]
    fn tkv_general_test() {
        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            let transaction_id = start_transaction(&mut tkv).unwrap();
            let res = in_transaction_set(
                "test_key".to_string(),
                "test_val".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            match res {
                Answer::TransactionalSetOk(_transactional_set_ok_answer) => {}
                _ => {
                    panic!("Expect Answer::SetOk, got other fields");
                }
            }
            let commit_transaction =
                Instruction::CommitTransaction(CommitTransactionInstruction { transaction_id });
            tkv.execute(&commit_transaction).unwrap();
            assert_eq!(tkv.vkv.get_current_height(), 1);
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            start_transaction(&mut tkv).unwrap();
            let res = start_transaction(&mut tkv);
            match res {
                Ok(_) => {}
                Err(_error) => {
                    panic!("Expect buffer transaction successful");
                }
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
                Err(_error) => {}
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
                Err(_error) => {}
            }
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            let transaction_id = start_transaction(&mut tkv).unwrap();
            in_transaction_set(
                "test_key".to_string(),
                "test_val".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            in_transaction_set(
                "test_key1".to_string(),
                "test_val1".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            in_transaction_set(
                "test_key2".to_string(),
                "test_val2".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            assert_eq!(tkv.executed_instructions.len(), 3);
            let abort_transaction =
                Instruction::AbortTransaction(AbortTransactionInstruction { transaction_id });
            tkv.execute(&abort_transaction).unwrap();
            assert_eq!(tkv.height_before_transaction, 0);
            assert_eq!(tkv.executed_instructions.len(), 0);

            match atomic_get_string("test_key".to_string(), &mut tkv) {
                Ok(_) => {
                    panic!("Should not get any data after execute abort.");
                }
                Err(_) => {}
            }
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);

            let transaction_id = start_transaction(&mut tkv).unwrap();
            in_transaction_set(
                "test_key".to_string(),
                "test_val1".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            in_transaction_set(
                "test_key".to_string(),
                "test_val2".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            in_transaction_set(
                "test_key".to_string(),
                "test_val3".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            in_transaction_revert("test_key".to_string(), 1, &mut tkv, transaction_id).unwrap();
            let commit_transaction =
                Instruction::CommitTransaction(CommitTransactionInstruction { transaction_id });
            tkv.execute(&commit_transaction).unwrap();

            let res = atomic_get_string("test_key".to_string(), &mut tkv).unwrap();
            assert_eq!(res, "test_val1".to_string());
            assert_eq!(tkv.vkv.get_current_height(), 4);
        }

        {
            let mut tkv = init_tkv();
            assert_eq!(tkv.height_before_transaction, 0);
            let transaction_id = start_transaction(&mut tkv).unwrap();
            in_transaction_set(
                "test_key".to_string(),
                "test_val1".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            in_transaction_set(
                "test_key".to_string(),
                "test_val2".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            in_transaction_set(
                "test_key".to_string(),
                "test_val3".to_string(),
                transaction_id,
                &mut tkv,
            )
            .unwrap();
            in_transaction_revert_all(2, transaction_id, &mut tkv).unwrap();

            let commit_transaction =
                Instruction::CommitTransaction(CommitTransactionInstruction { transaction_id });
            tkv.execute(&commit_transaction).unwrap();

            let res = atomic_get_string("test_key".to_string(), &mut tkv).unwrap();
            assert_eq!(res, "test_val2".to_string());
        }
    }

    #[test]
    fn test_serializable_transaction() {
        {
            let mut tkv = init_tkv();
            let start_transaction_instruction = get_start_transaction_insturction();
            match tkv.execute(&start_transaction_instruction) {
                Ok(answer) => match answer {
                    Answer::StartTransactionOk(start_transaction_ok) => {
                        assert_eq!(start_transaction_ok.transaction_id, 2);
                    }
                    _ => panic!("Should get transaction start ok"),
                },
                Err(_error) => {
                    panic!("Should get transaction start ok");
                }
            };
            match tkv.execute(&start_transaction_instruction) {
                Ok(answer) => match answer {
                    Answer::AppendTransactionOk(append_transaction_ok) => {
                        assert_eq!(append_transaction_ok.transaction_id, 3)
                    }
                    _ => {
                        panic!("Should buffer transaction ok");
                    }
                },
                Err(_error) => {
                    panic!("Should buffer transaction ok");
                }
            };
            let transactional_set_instruction = get_transactional_set_instruction(
                "test_key".to_string(),
                "test_val".to_string(),
                2,
            );
            match tkv.execute(&transactional_set_instruction) {
                Ok(answer) => match answer {
                    Answer::TransactionalSetOk(_set_ok_answer) => {}
                    _ => panic!("Should get transactional set ok answer"),
                },
                Err(_error) => panic!("Should get transactional set ok answer"),
            };
            let commit_transaction =
                Instruction::CommitTransaction(CommitTransactionInstruction { transaction_id: 2 });
            match tkv.execute(&commit_transaction) {
                Ok(answer) => match answer {
                    Answer::CommitTransactionOk(commit_ok_answer) => {
                        assert_eq!(commit_ok_answer.commited_transaction_id, 2);
                        assert_eq!(commit_ok_answer.next_active_transaction_id.unwrap(), 3);
                    }
                    _ => panic!("Should get commit transaction ok answer"),
                },
                Err(_error) => panic!("Should get commit transaction ok answer"),
            };
        }

        {
            let mut tkv = init_tkv();
            let start_transaction_instruction = get_start_transaction_insturction();
            match tkv.execute(&start_transaction_instruction) {
                Ok(answer) => match answer {
                    Answer::StartTransactionOk(start_transaction_ok) => {
                        assert_eq!(start_transaction_ok.transaction_id, 2);
                    }
                    _ => panic!("Should get transaction start ok"),
                },
                Err(_error) => {
                    panic!("Should get transaction start ok");
                }
            };

            match tkv.execute(&start_transaction_instruction) {
                Ok(answer) => match answer {
                    Answer::AppendTransactionOk(append_transaction_ok) => {
                        assert_eq!(append_transaction_ok.transaction_id, 3)
                    }
                    _ => {
                        panic!("Should buffer transaction ok");
                    }
                },
                Err(_error) => {
                    panic!("Should buffer transaction ok");
                }
            };

            let transactional_set_instruction = get_transactional_set_instruction(
                "test_key".to_string(),
                "test_val".to_string(),
                3,
            );
            match tkv.execute(&transactional_set_instruction) {
                Ok(answer) => match answer {
                    _ => panic!("Should get transactional set ok answer"),
                },
                Err(_error) => {}
            };
        }
    }
}
