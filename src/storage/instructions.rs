use serde::{Deserialize, Serialize};

use crate::declarations::basics::{
    BoxedStoreKey, BoxedStoreValue, StoreKey, StoreKeyFragment, StoreValue,
};
use crate::storage::kv::KVNamespace;
use crate::storage::tkv::TransactionId;
use crate::storage::vkv::{ChainHeight, UnitJournal};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetTargetSpec {
    pub key: StoreKey,
    pub value: StoreValue,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StoreNamespace(Vec<u8>);

impl StoreNamespace {
    pub fn new(ns: &[u8]) -> Self {
        StoreNamespace(ns.to_vec())
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Into<Vec<u8>> for StoreNamespace {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl From<KVNamespace> for StoreNamespace {
    fn from(ns: KVNamespace) -> Self {
        Self(ns.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertTargetSpec {
    pub key: StoreKey,
    pub height: ChainHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitTransactionInstruction {
    pub transaction_id: TransactionId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AbortTransactionInstruction {
    pub transaction_id: TransactionId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GetManyTargetSpec {
    Keys(Vec<StoreKey>),
    KeyPrefix(StoreKeyFragment),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetManyInstruction {
    pub height: Option<ChainHeight>,
    pub targets: GetManyTargetSpec,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetOneInstruction {
    pub height: Option<ChainHeight>,
    pub key: StoreKey,
}

impl From<GetOneInstruction> for Instruction {
    fn from(instruction: GetOneInstruction) -> Instruction {
        Instruction::Data(DataInstruction::Read(DataReadInstruction::GetOne(
            instruction,
        )))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetManyInstruction {
    pub targets: Vec<SetTargetSpec>,
}

impl From<SetManyInstruction> for Instruction {
    fn from(instruction: SetManyInstruction) -> Instruction {
        Instruction::Data(DataInstruction::Write(DataWriteInstruction::SetMany(
            instruction,
        )))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertManyInstruction {
    pub targets: Vec<RevertTargetSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertAllInstruction {
    pub target_height: ChainHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwitchNamespaceInstruction {
    pub new_namespace: StoreNamespace,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadNamespaceInstruction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetJournalInstruction {
    pub key: StoreKey,
}

impl From<GetJournalInstruction> for Instruction {
    fn from(instruction: GetJournalInstruction) -> Instruction {
        Instruction::Data(DataInstruction::Read(DataReadInstruction::GetJournal(
            instruction,
        )))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DataReadInstruction {
    GetOne(GetOneInstruction),
    GetMany(GetManyInstruction),
    GetJournal(GetJournalInstruction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DataWriteInstruction {
    SetMany(SetManyInstruction),
    RevertMany(RevertManyInstruction),
    RevertAll(RevertAllInstruction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DataInstruction {
    Read(DataReadInstruction),
    Write(DataWriteInstruction),
}

impl From<DataInstruction> for Instruction {
    fn from(instruction: DataInstruction) -> Instruction {
        Instruction::Data(instruction)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionalDataInstruction {
    pub plain_instruction: DataInstruction,
    pub transaction_id: TransactionId,
}

impl From<TransactionalDataInstruction> for Instruction {
    fn from(instruction: TransactionalDataInstruction) -> Self {
        Instruction::TransactionalData(instruction)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionMetaInstruction {
    StartTransaction,
    CommitTransaction(CommitTransactionInstruction),
    AbortTransaction(AbortTransactionInstruction),
}

impl From<TransactionMetaInstruction> for Instruction {
    fn from(instruction: TransactionMetaInstruction) -> Self {
        Instruction::TransactionMeta(instruction)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DBSystemInstruction {
    SwitchNamespace(SwitchNamespaceInstruction),
    ReadNamespace(ReadNamespaceInstruction),
}

impl From<DBSystemInstruction> for Instruction {
    fn from(instruction: DBSystemInstruction) -> Instruction {
        Instruction::DBSystem(instruction)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Instruction {
    Data(DataInstruction),
    TransactionalData(TransactionalDataInstruction),
    TransactionMeta(TransactionMetaInstruction),
    DBSystem(DBSystemInstruction),
}

#[derive(Debug)]
pub struct StartTransactionOkAnswer {
    pub transaction_id: TransactionId,
}

#[derive(Debug)]
pub struct TransactionPendingAnswer {
    pub transaction_id: TransactionId,
}

#[derive(Debug)]
pub struct CommitTransactionOkAnswer {
    pub committed_transaction_id: TransactionId,
    pub next_active_transaction_id: Option<TransactionId>,
}

#[derive(Debug)]
pub struct AbortTransactionOkAnswer {
    pub transaction_id: TransactionId,
}

#[derive(Debug)]
pub struct GetManyOkAnswer {
    pub data: Vec<(BoxedStoreKey, BoxedStoreValue)>,
}

#[derive(Debug)]
pub struct GetOneOkAnswer {
    pub value: StoreValue,
}

#[derive(Debug)]
pub struct SetOkAnswer {
    pub count: usize,
}

#[derive(Debug)]
pub struct RevertOkAnswer {}

#[derive(Debug)]
pub struct RevertAllOkAnswer {
    pub reverted_keys: Vec<StoreKey>,
}

#[derive(Debug)]
pub struct SwitchNamespaceOkAnswer {
    pub new_namespace: StoreNamespace,
}

#[derive(Debug)]
pub struct ReadNamespaceOkAnswer {
    pub namespace: StoreNamespace,
}

#[derive(Debug)]
pub struct GetJournalOkAnswer {
    pub journal: UnitJournal,
}

#[derive(Debug)]
pub enum DataReadAnswer {
    GetManyOk(GetManyOkAnswer),
    GetOneOk(GetOneOkAnswer),
    GetJournalOk(GetJournalOkAnswer),
}

#[derive(Debug)]
pub enum DataWriteAnswer {
    SetOk(SetOkAnswer),
    RevertOk(RevertOkAnswer),
    RevertAllOk(RevertAllOkAnswer),
}

#[derive(Debug)]
pub enum DataAnswer {
    Read(DataReadAnswer),
    Write(DataWriteAnswer),
}

#[derive(Debug)]
pub struct TransactionalDataAnswer {
    pub transaction_id: TransactionId,
    pub answer: DataAnswer,
}

#[derive(Debug)]
pub enum TransactionMetaAnswer {
    StartTransactionOk(StartTransactionOkAnswer),
    AppendTransactionOk(TransactionPendingAnswer),
    CommitTransactionOk(CommitTransactionOkAnswer),
    AbortTransactionOk(AbortTransactionOkAnswer),
}

#[derive(Debug)]
pub enum DBSystemAnswer {
    SwitchNamespaceOk(SwitchNamespaceOkAnswer),
    ReadNamespaceOk(ReadNamespaceOkAnswer),
}

#[derive(Debug)]
pub enum Answer {
    DataAccess(DataAnswer),
    TransactionalData(TransactionalDataAnswer),
    TransactionMeta(TransactionMetaAnswer),
    DBSystem(DBSystemAnswer),
}
