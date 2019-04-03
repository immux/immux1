// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#standard-message-header

use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::opcodes::MongoOpCode;
use crate::cortices::mongo::utils::{parse_u32, pick_op_code};

#[derive(Debug)]
pub struct MsgHeader {
    // total message size, including this
    pub message_length: u32,

    // identifier for this message
    pub request_id: u32,

    // requestID from the original request (used in responses from db)
    pub response_to: u32,

    // request type
    pub op_code: MongoOpCode,
}

pub fn parse_msg_header(buffer: &[u8]) -> Result<(MsgHeader, &[u8]), MongoParserError> {
    let (message_length, next_buffer) = parse_u32(buffer)?;
    let (request_id, next_buffer) = parse_u32(next_buffer)?;
    let (response_to, next_buffer) = parse_u32(next_buffer)?;
    let (op_code_u32, next_buffer) = parse_u32(next_buffer)?;
    let op_code = pick_op_code(op_code_u32)?;
    Ok((
        MsgHeader {
            message_length,
            request_id,
            response_to,
            op_code,
        },
        next_buffer,
    ))
}
