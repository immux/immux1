use serde::{Deserialize, Serialize};

use crate::storage::vkv::{Entry, InstructionHeight};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTargetSpec {
    pub key: Vec<u8>,
    pub height: Option<InstructionHeight>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetTargetSpec {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertTargetSpec {
    pub key: Vec<u8>,
    pub height: InstructionHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitTransactionInstruction {
    pub transaction_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AbortTransactionInstruction {
    pub transaction_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtomicGetInstruction {
    pub targets: Vec<GetTargetSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtomicGetOneInstruction {
    pub target: GetTargetSpec,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InTransactionGetInstruction {
    pub targets: Vec<GetTargetSpec>,
    pub transaction_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InTransactionGetOneInstruction {
    pub target: GetTargetSpec,
    pub transaction_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtomicSetInstruction {
    pub targets: Vec<SetTargetSpec>,
    pub increment_height: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InTransactionSetInstruction {
    pub targets: Vec<SetTargetSpec>,
    pub transaction_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtomicRevertInstruction {
    pub targets: Vec<RevertTargetSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InTransactionRevertInstruction {
    pub targets: Vec<RevertTargetSpec>,
    pub transaction_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtomicRevertAllInstruction {
    pub target_height: InstructionHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InTransactionRevertAllInstruction {
    pub target_height: InstructionHeight,
    pub transaction_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwitchNamespaceInstruction {
    pub new_namespace: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadNamespaceInstruction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetEntryInstruction {
    pub key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Instruction {
    StartTransaction,
    CommitTransaction(CommitTransactionInstruction),
    AbortTransaction(AbortTransactionInstruction),
    InTransactionGet(InTransactionGetInstruction),
    InTransactionGetOne(InTransactionGetOneInstruction),
    InTransactionSet(InTransactionSetInstruction),
    InTransactionRevert(InTransactionRevertInstruction),
    InTransactionRevertAll(InTransactionRevertAllInstruction),
    AtomicGet(AtomicGetInstruction),
    AtomicSet(AtomicSetInstruction),
    AtomicGetOne(AtomicGetOneInstruction),
    AtomicRevert(AtomicRevertInstruction),
    AtomicRevertAll(AtomicRevertAllInstruction),
    SwitchNamespace(SwitchNamespaceInstruction),
    ReadNamespace(ReadNamespaceInstruction),
    GetEntry(GetEntryInstruction),
}

#[derive(Debug)]
pub struct StartTransactionOkAnswer {
    pub transaction_id: u64,
}

#[derive(Debug)]
pub struct TransactionPendingAnswer {
    pub transaction_id: u64,
}

#[derive(Debug)]
pub struct CommitTransactionOkAnswer {
    pub commited_transaction_id: u64,
    pub next_active_transaction_id: Option<u64>,
}

#[derive(Debug)]
pub struct AbortTransactionOkAnswer {
    pub transaction_id: u64,
}

#[derive(Debug)]
pub struct GetOkAnswer {
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct GetOneOkAnswer {
    pub item: Vec<u8>,
}

#[derive(Debug)]
pub struct TransactionalGetOkAnswer {
    pub transaction_id: u64,
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct TransactionalGetOneOkAnswer {
    pub transaction_id: u64,
    pub item: Vec<u8>,
}

#[derive(Debug)]
pub struct SetOkAnswer {
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct TransactionalSetOkAnswer {
    pub transaction_id: u64,
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct RevertOkAnswer {
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct TransactionalRevertOkAnswer {
    pub transaction_id: u64,
    pub items: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct RevertAllOkAnswer {
    pub reverted_keys: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct TransactionalRevertAllOkAnswer {
    pub transaction_id: u64,
    pub reverted_keys: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct SwitchNamespaceOkAnswer {
    pub new_namespace: Vec<u8>,
}

#[derive(Debug)]
pub struct ReadNamespaceOkAnswer {
    pub namespace: Vec<u8>,
}

#[derive(Debug)]
pub struct GetEntryOkAnswer {
    pub entry: Entry,
}

#[derive(Debug)]
pub enum Answer {
    StartTransactionOk(StartTransactionOkAnswer),
    AppendTransactionOk(TransactionPendingAnswer),
    CommitTransactionOk(CommitTransactionOkAnswer),
    AbortTransactionOk(AbortTransactionOkAnswer),
    TransactionalGetOk(TransactionalGetOkAnswer),
    TransactionalGetOneOk(TransactionalGetOneOkAnswer),
    TransactionalSetOk(TransactionalSetOkAnswer),
    TransactionalRevertOk(TransactionalRevertOkAnswer),
    TransactionalRevertAllOk(TransactionalRevertAllOkAnswer),
    GetOk(GetOkAnswer),
    GetOneOk(GetOneOkAnswer),
    SetOk(SetOkAnswer),
    RevertOk(RevertOkAnswer),
    RevertAllOk(RevertAllOkAnswer),
    SwitchNamespaceOk(SwitchNamespaceOkAnswer),
    ReadNamespaceOk(ReadNamespaceOkAnswer),
    GetEntryOk(GetEntryOkAnswer),
}
