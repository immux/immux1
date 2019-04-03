use std::ffi::CString;
use std::mem::size_of;

use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::opcodes::{
    MongoOpCode, MONGO_OP_COMMAND_CODE, MONGO_OP_COMMAND_REPLY_CODE, MONGO_OP_DELETE_CODE,
    MONGO_OP_GET_MORE_CODE, MONGO_OP_INSERT_CODE, MONGO_OP_KILL_CURSORS_CODE, MONGO_OP_MSG_CODE,
    MONGO_OP_QUERY_CODE, MONGO_OP_REPLY_CODE, MONGO_OP_UPDATE_CODE,
};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::u8_array_to_u32;

pub fn pick_op_code(op: u32) -> Result<MongoOpCode, MongoParserError> {
    match op {
        MONGO_OP_REPLY_CODE => Ok(MongoOpCode::OpReply),
        MONGO_OP_UPDATE_CODE => Ok(MongoOpCode::OpUpdate),
        MONGO_OP_INSERT_CODE => Ok(MongoOpCode::OpInsert),
        MONGO_OP_QUERY_CODE => Ok(MongoOpCode::OpQuery),
        MONGO_OP_GET_MORE_CODE => Ok(MongoOpCode::OpGetMore),
        MONGO_OP_DELETE_CODE => Ok(MongoOpCode::OpDelete),
        MONGO_OP_KILL_CURSORS_CODE => Ok(MongoOpCode::OpKillCursors),
        MONGO_OP_COMMAND_CODE => Ok(MongoOpCode::OpCommand),
        MONGO_OP_COMMAND_REPLY_CODE => Ok(MongoOpCode::OpCommandReply),
        MONGO_OP_MSG_CODE => Ok(MongoOpCode::OpMsg),
        _ => Err(MongoParserError::UnknownOpCode(op)),
    }
}

pub fn parse_cstring(buffer: &[u8]) -> Result<(CString, &[u8]), MongoParserError> {
    match buffer.iter().position(|&r| r == b'\0') {
        None => Err(MongoParserError::NoZeroTrailingInCstringBuffer),
        Some(terminal_index) => {
            let new_value = if terminal_index == 0 {
                CString::new("")
            } else {
                CString::new(&buffer[..terminal_index])
            };
            match new_value {
                Err(_nulerror) => return Err(MongoParserError::CstringContainZeroByte),
                Ok(value) => {
                    let remaining_buffer = &buffer[terminal_index + 1..];
                    return Ok((value, remaining_buffer));
                }
            }
        }
    }
}

pub fn parse_bson_document(buffer: &[u8]) -> Result<(bson::Document, &[u8]), MongoParserError> {
    let (bson_size, _next_buffer) = parse_u32(buffer)?;
    match bson::decode_document(&mut &(*buffer)[0..(bson_size as usize)]) {
        Err(error) => Err(MongoParserError::ParseBsonError(error)),
        Ok(bson_document) => Ok((bson_document, &buffer[(bson_size as usize)..])),
    }
}

pub fn parse_field<'a, Value, E>(
    buffer: &'a [u8],
    error: MongoParserError,
    extract_fn: &Fn(&[u8]) -> Result<Value, E>,
) -> Result<(Value, &'a [u8]), MongoParserError> {
    let field_size = size_of::<Value>();
    if field_size > buffer.len() {
        eprintln!("Buffer doesn't have enough size for slicing");
        return Err(MongoParserError::NotEnoughBufferSize);
    }
    let next_buffer = &buffer[field_size..];
    match extract_fn(&buffer[0..field_size]) {
        Ok(val) => Ok((val, next_buffer)),
        Err(_) => Err(error),
    }
}

fn read_u32(input: &[u8]) -> UnumResult<u32> {
    if input.len() < size_of::<u32>() {
        Err(UnumError::MongoParser(
            MongoParserError::NotEnoughBufferSize,
        ))
    } else {
        Ok(u8_array_to_u32(&[input[0], input[1], input[2], input[3]]))
    }
}

pub fn parse_u32(buffer: &[u8]) -> Result<(u32, &[u8]), MongoParserError> {
    parse_field(buffer, MongoParserError::NotEnoughBufferSize, &read_u32)
}
