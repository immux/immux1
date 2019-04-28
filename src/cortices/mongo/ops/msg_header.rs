// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#standard-message-header

use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::opcodes::MongoOpCode;
use crate::cortices::mongo::utils::{get_op_code_value, pick_op_code};
use crate::cortices::utils::parse_u32;
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::u32_to_u8_array;

#[derive(Debug, Clone)]
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

pub fn parse_msg_header(buffer: &[u8]) -> UnumResult<(MsgHeader, usize)> {
    let mut index: usize = 0;
    let (message_length, offset) = parse_u32(
        &buffer[index..],
        UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
    )?;
    index += offset;
    let (request_id, offset) = parse_u32(
        &buffer[index..],
        UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
    )?;
    index += offset;
    let (response_to, offset) = parse_u32(
        &buffer[index..],
        UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
    )?;
    index += offset;
    let (op_code_u32, offset) = parse_u32(
        &buffer[index..],
        UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
    )?;
    index += offset;
    let op_code = pick_op_code(op_code_u32)?;
    Ok((
        MsgHeader {
            message_length,
            request_id,
            response_to,
            op_code,
        },
        index,
    ))
}

pub fn serialize_msg_header(message_header: &MsgHeader) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    res.append(&mut u32_to_u8_array(message_header.message_length).to_vec());
    res.append(&mut u32_to_u8_array(message_header.request_id).to_vec());
    res.append(&mut u32_to_u8_array(message_header.response_to).to_vec());
    res.append(&mut u32_to_u8_array(get_op_code_value(&message_header.op_code)).to_vec());
    res
}
