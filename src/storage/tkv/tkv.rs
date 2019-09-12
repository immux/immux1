use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::storage::instructions::{Answer, Instruction, StoreNamespace};
use crate::storage::kv::{KeyValueEngine};
use crate::storage::vkv::{ImmuxDBVersionedKeyValueStore, VersionedKeyValueStore};

#[derive(Debug)]
pub enum TransactionError {
    TransactionInProgress,
    TransactionNotStarted,
    UnexpectedAnswer,
    AbortInstructionError,
    CannotSwitchNamespaceWhileTransactionIsOngoing,
}

pub trait TransactionKeyValueStore {
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, ImmuxError>;
}

pub struct ImmuxDBTransactionKeyValueStore {
    vkv: ImmuxDBVersionedKeyValueStore,
}

impl ImmuxDBTransactionKeyValueStore {
    pub fn new(
        engine_choice: &KeyValueEngine,
        data_root: &str,
        namespace: &StoreNamespace,
    ) -> Result<ImmuxDBTransactionKeyValueStore, ImmuxError> {
        let vkv = ImmuxDBVersionedKeyValueStore::new(engine_choice, data_root, namespace)?;
        let tkv = ImmuxDBTransactionKeyValueStore { vkv };
        return Ok(tkv);
    }

    fn pass_to_vkv(&mut self, instruction: &Instruction) -> ImmuxResult<Answer> {
        self.vkv.execute(instruction)
    }
}

impl TransactionKeyValueStore for ImmuxDBTransactionKeyValueStore {
    fn execute(&mut self, instruction: &Instruction) -> Result<Answer, ImmuxError> {
        self.pass_to_vkv(instruction)
    }
}

#[cfg(test)]
mod tkv_tests {
    use crate::declarations::errors::ImmuxResult;
    use crate::storage::instructions::{
        Answer, CommitTransactionInstruction, DataAnswer, DataWriteAnswer, Instruction,
        TransactionMetaAnswer, TransactionMetaInstruction, TransactionalDataAnswer,
    };
    use crate::storage::tkv::tkv::tkv_test_utils::{
        get_start_transaction_instruction, TKVTestCore,
    };
    use crate::storage::tkv::transaction_id::TransactionId;
    use crate::storage::tkv::TransactionKeyValueStore;

    #[test]
    #[ignore]
    fn tkv_start_transaction() {
        let mut core = TKVTestCore::new("tkv_start_transaction");
        core.start_transaction().unwrap();
    }

    #[test]
    #[ignore]
    fn tkv_set_answer_type() {
        let mut core = TKVTestCore::new("tkv_set_answer_type");
        let (tid_int, _) = core.start_transaction().unwrap();
        let answer = core
            .transactional_set("test_key", "test_val", tid_int)
            .unwrap();
        match answer {
            Answer::TransactionalData(TransactionalDataAnswer {
                answer: DataAnswer::Write(DataWriteAnswer::SetOk(_answer)),
                ..
            }) => {}
            _ => {
                panic!("Expect Transactional Set Ok answer, got something else instead");
            }
        }
        core.commit_transaction(tid_int).unwrap();
    }

    #[test]
    #[ignore]
    #[should_panic]
    fn tkv_commit_transaction_not_started() {
        let mut core = TKVTestCore::new("tkv_commit_transaction_not_started");
        let transaction_id = 0;
        core.commit_transaction(transaction_id).unwrap();
    }

    #[test]
    #[ignore]
    #[should_panic]
    fn tkv_abort_transaction_not_started() {
        let mut core = TKVTestCore::new("tkv_abort_transaction_not_started");
        let transaction_id = 0;
        core.abort_transaction(transaction_id).unwrap();
    }

    #[test]
    #[ignore]
    fn test_abort_transaction() {
        let key = "test_key";
        let mut core = TKVTestCore::new("test_abort_transaction");
        let (transaction_id, _) = core.start_transaction().unwrap();
        core.transactional_set(key, "test_value", transaction_id)
            .unwrap();
        core.abort_transaction(transaction_id).unwrap();
        match core.simple_get(key) {
            Ok(None) => {}
            Ok(Some(value)) => panic!("Values should not be accessible, got: {}", value),
            Err(err) => panic!("error: {}", err),
        }
    }

    #[test]
    #[ignore]
    fn test_revert_one() -> ImmuxResult<()> {
        let mut core = TKVTestCore::new("test_revert_one");
        let (tid, _) = core.start_transaction()?;
        core.transactional_set("test_key", "test_value1", tid)?;
        core.transactional_set("test_key", "test_value2", tid)?;
        core.transactional_set("test_key", "test_value3", tid)?;
        core.transactional_revert("test_key", 1, tid)?;
        core.commit_transaction(tid)?;

        let current_value = core.simple_get("test_key").unwrap();
        assert_eq!(current_value, Some("test_value1".to_string()));
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_revert_all() -> ImmuxResult<()> {
        let mut core = TKVTestCore::new("test_revert_all");
        let (tid, _) = core.start_transaction()?;
        core.transactional_set("test_key", "test_value1", tid)?;
        core.transactional_set("test_key", "test_value2", tid)?;
        core.transactional_set("test_key", "test_value3", tid)?;
        core.transactional_revert_all(2, tid)?;
        core.commit_transaction(tid)?;

        let current_value = core.simple_get("test_key")?;
        assert_eq!(current_value, Some("test_value2".to_string()));
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_multiple_concurrent_transactions() -> ImmuxResult<()> {
        let mut core = TKVTestCore::new("test_multiple_concurrent_transactions");

        let _start_transaction = get_start_transaction_instruction();

        let (tid1, answer1) = core.start_transaction()?;
        let (tid2, answer2) = core.start_transaction()?;

        match answer1 {
            TransactionMetaAnswer::StartTransactionOk(_) => {}
            _ => panic!("Incorrect answer type for the first transaction"),
        }
        match answer2 {
            TransactionMetaAnswer::AppendTransactionOk(_) => {}
            _ => panic!("Incorrect answer type for the subsequent transaction"),
        }

        core.transactional_set("test_key", "test_val", tid1)?;

        let commit_transaction = Instruction::TransactionMeta(
            TransactionMetaInstruction::CommitTransaction(CommitTransactionInstruction {
                transaction_id: TransactionId::new(tid1),
            }),
        );
        match core.tkv.execute(&commit_transaction) {
            Err(_error) => panic!("Cannot commit"),
            Ok(Answer::TransactionMeta(TransactionMetaAnswer::CommitTransactionOk(
                commit_ok_answer,
            ))) => {
                assert_eq!(commit_ok_answer.committed_transaction_id.as_int(), tid1);
                assert_eq!(
                    commit_ok_answer
                        .next_active_transaction_id
                        .unwrap()
                        .as_int(),
                    tid2
                );
            }
            Ok(_answer) => panic!("Wrong answer type"),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tkv_test_utils {
    use crate::config::{ImmuxDBConfiguration, DEFAULT_PERMANENCE_PATH};
    use crate::declarations::basics::{StoreKey, StoreValue};
    use crate::declarations::errors::{ImmuxError, ImmuxResult};
    use crate::storage::instructions::{
        AbortTransactionInstruction, Answer, CommitTransactionInstruction, DataAnswer,
        DataInstruction, DataReadAnswer, DataReadInstruction, DataWriteInstruction,
        GetOneInstruction, Instruction, RevertAllInstruction, RevertManyInstruction,
        RevertTargetSpec, SetManyInstruction, SetTargetSpec, StoreNamespace, TransactionMetaAnswer,
        TransactionMetaInstruction, TransactionalDataInstruction,
    };
    use crate::storage::tkv::transaction_id::TransactionId;
    use crate::storage::tkv::{
        ImmuxDBTransactionKeyValueStore, TransactionError, TransactionKeyValueStore,
    };
    use crate::storage::vkv::{ChainHeight, VkvError};
    use crate::utils::utf8_to_string;

    pub struct TKVTestCore {
        pub tkv: ImmuxDBTransactionKeyValueStore,
    }

    impl TKVTestCore {
        pub fn new(ns: &str) -> TKVTestCore {
            let config = ImmuxDBConfiguration::default();
            let namespace = StoreNamespace::new(ns.as_bytes());
            let tkv = ImmuxDBTransactionKeyValueStore::new(
                &config.engine_choice,
                DEFAULT_PERMANENCE_PATH,
                &namespace,
            )
            .unwrap();
            return TKVTestCore { tkv };
        }
        pub fn start_transaction(&mut self) -> ImmuxResult<(u64, TransactionMetaAnswer)> {
            let start_transaction_instruction = get_start_transaction_instruction();
            match self.tkv.execute(&start_transaction_instruction) {
                Err(error) => {
                    return Err(error);
                }
                Ok(answer) => match answer {
                    Answer::TransactionMeta(TransactionMetaAnswer::StartTransactionOk(
                        start_transaction_ok_answer,
                    )) => {
                        let tid = start_transaction_ok_answer.transaction_id.as_int();
                        return Ok((
                            tid,
                            TransactionMetaAnswer::StartTransactionOk(start_transaction_ok_answer),
                        ));
                    }
                    Answer::TransactionMeta(TransactionMetaAnswer::AppendTransactionOk(
                        append_transaction_ok_answer,
                    )) => {
                        let tid = append_transaction_ok_answer.transaction_id.as_int();
                        return Ok((
                            tid,
                            TransactionMetaAnswer::AppendTransactionOk(
                                append_transaction_ok_answer,
                            ),
                        ));
                    }
                    _ => {
                        return Err(ImmuxError::Transaction(TransactionError::UnexpectedAnswer));
                    }
                },
            }
        }
        pub fn transactional_set(
            &mut self,
            key_str: &str,
            value_str: &str,
            tid_int: u64,
        ) -> Result<Answer, ImmuxError> {
            let key = StoreKey::from(key_str.as_bytes().to_vec());
            let value = StoreValue::new(Some(value_str.as_bytes().to_vec()));
            let instruction =
                get_transactional_set_instruction(key, value, TransactionId::new(tid_int));
            return self.tkv.execute(&instruction);
        }
        pub fn simple_get(&mut self, key_str: &str) -> Result<Option<String>, ImmuxError> {
            let key = StoreKey::from(key_str.as_bytes().to_vec());
            let get = Instruction::Data(DataInstruction::Read(DataReadInstruction::GetOne(
                GetOneInstruction { height: None, key },
            )));
            match self.tkv.execute(&get) {
                Err(ImmuxError::VKV(VkvError::MissingJournal(_))) => Ok(None),
                Err(error) => return Err(error),
                Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                    match answer.value.inner() {
                        None => Ok(None),
                        Some(data) => Ok(Some(utf8_to_string(data))),
                    }
                }
                Ok(_answer) => {
                    return Err(ImmuxError::Transaction(TransactionError::UnexpectedAnswer))
                }
            }
        }
        pub fn transactional_revert_all(
            &mut self,
            target_height: u64,
            transaction_id: u64,
        ) -> Result<Answer, ImmuxError> {
            let instruction = get_transactional_revert_all_instruction(
                ChainHeight::new(target_height),
                TransactionId::new(transaction_id),
            );
            return self.tkv.execute(&instruction);
        }
        pub fn transactional_revert(
            &mut self,
            key_str: &str,
            height_u64: u64,
            transaction_id_int: u64,
        ) -> Result<Answer, ImmuxError> {
            let key = StoreKey::from(key_str.as_bytes().to_vec());
            let height = ChainHeight::new(height_u64);
            let transaction_id = TransactionId::new(transaction_id_int);
            let revert_target_spec = RevertTargetSpec { key, height };
            let instruction = Instruction::TransactionalData(TransactionalDataInstruction {
                plain_instruction: DataInstruction::Write(DataWriteInstruction::RevertMany(
                    RevertManyInstruction {
                        targets: vec![revert_target_spec],
                    },
                )),
                transaction_id,
            });
            return self.tkv.execute(&instruction);
        }
        pub fn commit_transaction(&mut self, id: u64) -> ImmuxResult<Answer> {
            let commit_transaction = Instruction::TransactionMeta(
                TransactionMetaInstruction::CommitTransaction(CommitTransactionInstruction {
                    transaction_id: TransactionId::new(id),
                }),
            );
            self.tkv.execute(&commit_transaction)
        }
        pub fn abort_transaction(&mut self, id: u64) -> ImmuxResult<Answer> {
            let abort_transaction = Instruction::TransactionMeta(
                TransactionMetaInstruction::AbortTransaction(AbortTransactionInstruction {
                    transaction_id: TransactionId::new(id),
                }),
            );
            self.tkv.execute(&abort_transaction)
        }
    }

    pub fn get_start_transaction_instruction() -> Instruction {
        return Instruction::TransactionMeta(TransactionMetaInstruction::StartTransaction);
    }

    pub fn get_transactional_set_instruction(
        key: StoreKey,
        value: StoreValue,
        transaction_id: TransactionId,
    ) -> Instruction {
        let set_target_spec = SetTargetSpec { key, value };
        return Instruction::TransactionalData(TransactionalDataInstruction {
            transaction_id,
            plain_instruction: DataInstruction::Write(DataWriteInstruction::SetMany(
                SetManyInstruction {
                    targets: vec![set_target_spec],
                },
            )),
        });
    }

    pub fn get_transactional_revert_all_instruction(
        target_height: ChainHeight,
        transaction_id: TransactionId,
    ) -> Instruction {
        let inner_transaction =
            DataInstruction::Write(DataWriteInstruction::RevertAll(RevertAllInstruction {
                target_height,
            }));
        return Instruction::TransactionalData(TransactionalDataInstruction {
            plain_instruction: inner_transaction,
            transaction_id,
        });
    }

}
