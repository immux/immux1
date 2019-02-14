use crate::storage::vkv::CommitHeight;

pub struct GetKeyQuery {
    pub key: Vec<u8>,
}

pub struct GetKeyAtHeightQuery {
    pub key: Vec<u8>,
    pub height: CommitHeight,
}

pub struct SetKeyQuery {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

pub struct RevertByKeyQuery {
    pub key: Vec<u8>,
    pub height: CommitHeight,
}

pub struct RevertAllQuery {
    pub height: CommitHeight,
}

pub enum Query {
    GetKey(GetKeyQuery),
    SetKey(SetKeyQuery),
    GetKeyAtHeight(GetKeyAtHeightQuery),
    RevertAll(RevertAllQuery),
    RevertByKey(RevertByKeyQuery),
}

#[derive(Debug)]
pub struct QueryResponse {
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct QueryError {
    pub error: String,
}

pub type QueryReturns = Result<QueryResponse, QueryError>;
