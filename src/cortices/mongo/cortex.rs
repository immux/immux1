use crate::cortices::mongo::ops::op_reply::serialize_op_reply;
use crate::cortices::mongo::parser::parse_mongo_incoming_bytes;
use crate::cortices::mongo::transformer::{transform_answer_for_mongo, transform_mongo_op};
use crate::declarations::errors::UnumResult;
use crate::storage::core::{CoreStore, UnumCore};
use crate::utils::pretty_dump;

pub fn mongo_cortex(bytes: &[u8], core: &mut UnumCore) -> UnumResult<Option<Vec<u8>>> {
    println!("Incoming bytes {}", bytes.len());
    pretty_dump(bytes);
    let op = parse_mongo_incoming_bytes(bytes)?;
    let instruction = transform_mongo_op(&op)?;
    let answer = core.execute(&instruction)?;
    let op_reply = transform_answer_for_mongo(&answer)?;
    let reply_bytes = serialize_op_reply(&op_reply)?;
    return Ok(Some(reply_bytes));
}
