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

pub fn parse_u8(buffer: &[u8]) -> UnumResult<(u8, &[u8])> {
    match buffer.first() {
        Some(val) => {
            return Ok((*val, &buffer[size_of::<u8>()..]));
        }
        None => Err(UnumError::MongoParser(
            MongoParserError::NotEnoughBufferSize,
        )),
    }
}

pub fn parse_cstring(buffer: &[u8]) -> UnumResult<(CString, usize, &[u8])> {
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
                    let remaining_buffer = &buffer[terminal_index + 1..];
                    return Ok((value, terminal_index + 1, remaining_buffer));
                }
            }
        }
    }
}

pub fn parse_bson_document(buffer: &[u8]) -> UnumResult<(bson::Document, &[u8])> {
    let (bson_size, _next_buffer) = parse_u32(buffer)?;
    match bson::decode_document(&mut &(*buffer)[0..(bson_size as usize)]) {
        Err(error) => Err(UnumError::MongoParser(MongoParserError::ParseBsonError(
            error,
        ))),
        Ok(bson_document) => Ok((bson_document, &buffer[(bson_size as usize)..])),
    }
}

pub fn parse_u32(buffer: &[u8]) -> UnumResult<(u32, &[u8])> {
    let field_size = size_of::<u32>();
    if buffer.len() < field_size {
        Err(UnumError::MongoParser(
            MongoParserError::NotEnoughBufferSize,
        ))
    } else {
        Ok((
            u8_array_to_u32(&[buffer[0], buffer[1], buffer[2], buffer[3]]),
            &buffer[field_size..],
        ))
    }
}

pub fn parse_u64(buffer: &[u8]) -> UnumResult<(u64, &[u8])> {
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
            &buffer[field_size..],
        ))
    }
}

#[cfg(test)]
mod utils_tests {
    use crate::cortices::mongo::utils::{parse_u32, parse_u64, parse_u8};

    #[test]
    fn test_parse_u8() {
        let buffer = [0x11];
        let res = parse_u8(&buffer);
        let (num, next_buffer) = res.unwrap();
        assert_eq!(17, num);
        assert_eq!(next_buffer.len(), 0);
    }

    #[test]
    fn test_parse_u8_error() {
        let buffer = [];
        match parse_u8(&buffer) {
            Ok(_) => {
                assert!(false, "Empty buffer should return an error");
            }
            Err(_) => {
                assert!(true);
            }
        }
    }

    #[test]
    fn test_parse_u64() {
        let res = parse_u64(&[0x0d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        let (num, next_buffer) = res.unwrap();
        assert_eq!(269, num);
        assert_eq!(next_buffer.len(), 0);
    }

    #[test]
    fn test_parse_u64_error() {
        match parse_u64(&[0x0d]) {
            Ok(_) => {
                assert!(false, "buffer size less then 8 should return an error");
            }
            Err(_) => {
                assert!(true);
            }
        }
    }

    #[test]
    fn test_parse_u32() {
        let res = parse_u32(&[0x0d, 0x01, 0x00, 0x00]);
        let (num, next_buffer) = res.unwrap();
        assert_eq!(269, num);
        assert_eq!(next_buffer.len(), 0);
    }

    #[test]
    fn test_parse_u32_error() {
        match parse_u32(&[0x0d]) {
            Ok(_) => {
                assert!(false, "buffer size less then 4 should return an error");
            }
            Err(_) => {
                assert!(true);
            }
        }
    }
}
