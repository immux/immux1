use std::ffi::CString;
use std::mem::size_of;

use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::opcodes::{
    MongoOpCode, MONGO_OP_COMMAND_CODE, MONGO_OP_COMMAND_REPLY_CODE, MONGO_OP_DELETE_CODE,
    MONGO_OP_GET_MORE_CODE, MONGO_OP_INSERT_CODE, MONGO_OP_KILL_CURSORS_CODE, MONGO_OP_MSG_CODE,
    MONGO_OP_QUERY_CODE, MONGO_OP_REPLY_CODE, MONGO_OP_UPDATE_CODE,
};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::{u8_array_to_u32, u8_array_to_u64};

pub fn pick_op_code(op: u32) -> UnumResult<MongoOpCode> {
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
        _ => Err(UnumError::MongoParser(MongoParserError::UnknownOpCode(op))),
    }
}

pub fn get_op_code_value(op_code: &MongoOpCode) -> u32 {
    match op_code {
        MongoOpCode::OpReply => MONGO_OP_REPLY_CODE,
        MongoOpCode::OpUpdate => MONGO_OP_UPDATE_CODE,
        MongoOpCode::OpInsert => MONGO_OP_INSERT_CODE,
        MongoOpCode::OpQuery => MONGO_OP_QUERY_CODE,
        MongoOpCode::OpGetMore => MONGO_OP_GET_MORE_CODE,
        MongoOpCode::OpDelete => MONGO_OP_DELETE_CODE,
        MongoOpCode::OpKillCursors => MONGO_OP_KILL_CURSORS_CODE,
        MongoOpCode::OpCommand => MONGO_OP_COMMAND_CODE,
        MongoOpCode::OpCommandReply => MONGO_OP_COMMAND_REPLY_CODE,
        MongoOpCode::OpMsg => MONGO_OP_MSG_CODE,
    }
}

pub fn parse_cstring(buffer: &[u8]) -> UnumResult<(CString, usize)> {
    match buffer.iter().position(|&r| r == b'\0') {
        None => Err(UnumError::MongoParser(
            MongoParserError::NoZeroTrailingInCstringBuffer,
        )),
        Some(terminal_index) => {
            let new_value = if terminal_index == 0 {
                CString::new("")
            } else {
                CString::new(&buffer[..terminal_index])
            };
            match new_value {
                Err(_nulerror) => {
                    return Err(UnumError::MongoParser(
                        MongoParserError::CstringContainZeroByte,
                    ));
                }
                Ok(value) => {
                    return Ok((value, terminal_index + 1));
                }
            }
        }
    }
}

pub fn parse_bson_document(buffer: &[u8]) -> UnumResult<(bson::Document, usize)> {
    let (bson_size, _next_buffer) = parse_u32(buffer)?;
    match bson::decode_document(&mut &(*buffer)[0..(bson_size as usize)]) {
        Err(error) => Err(UnumError::MongoParser(MongoParserError::ParseBsonError(
            error,
        ))),
        Ok(bson_document) => Ok((bson_document, bson_size as usize)),
    }
}

pub fn parse_u8(buffer: &[u8]) -> UnumResult<(u8, usize)> {
    let field_size = size_of::<u8>();
    if buffer.len() < field_size {
        Err(UnumError::MongoParser(
            MongoParserError::NotEnoughBufferSize,
        ))
    } else {
        Ok((buffer[0], field_size))
    }
}

pub fn parse_u32(buffer: &[u8]) -> UnumResult<(u32, usize)> {
    let field_size = size_of::<u32>();
    if buffer.len() < field_size {
        Err(UnumError::MongoParser(
            MongoParserError::NotEnoughBufferSize,
        ))
    } else {
        Ok((
            u8_array_to_u32(&[buffer[0], buffer[1], buffer[2], buffer[3]]),
            field_size,
        ))
    }
}

pub fn parse_u64(buffer: &[u8]) -> UnumResult<(u64, usize)> {
    let field_size = size_of::<u64>();
    if buffer.len() < field_size {
        Err(UnumError::MongoParser(
            MongoParserError::NotEnoughBufferSize,
        ))
    } else {
        Ok((
            u8_array_to_u64(&[
                buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                buffer[7],
            ]),
            field_size,
        ))
    }
}

#[cfg(test)]
mod mongo_utils_tests {
    use crate::cortices::mongo::utils::{parse_u32, parse_u64, parse_u8};

    #[test]
    fn test_parse_u8() {
        let buffer = [0x11];
        let res = parse_u8(&buffer);
        let (num, offset) = res.unwrap();
        assert_eq!(17, num);
        assert_eq!(offset, 1);
    }

    #[test]
    #[should_panic]
    fn test_parse_u8_error() {
        let buffer = [];
        parse_u8(&buffer).unwrap();
    }

    #[test]
    fn test_parse_u64() {
        let res = parse_u64(&[0x0d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        let (num, offset) = res.unwrap();
        assert_eq!(269, num);
        assert_eq!(offset, 8);
    }

    #[test]
    #[should_panic]
    fn test_parse_u64_error() {
        parse_u64(&[0x0d]).unwrap();
    }

    #[test]
    fn test_parse_u32() {
        let res = parse_u32(&[0x0d, 0x01, 0x00, 0x00]);
        let (num, offset) = res.unwrap();
        assert_eq!(269, num);
        assert_eq!(offset, 4);
    }

    #[test]
    #[should_panic]
    fn test_parse_u32_error() {
        parse_u32(&[0x0d]).unwrap();
    }
}
