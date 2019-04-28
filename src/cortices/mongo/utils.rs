use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::opcodes::{
    MongoOpCode, MONGO_OP_COMMAND_CODE, MONGO_OP_COMMAND_REPLY_CODE, MONGO_OP_DELETE_CODE,
    MONGO_OP_GET_MORE_CODE, MONGO_OP_INSERT_CODE, MONGO_OP_KILL_CURSORS_CODE, MONGO_OP_MSG_CODE,
    MONGO_OP_QUERY_CODE, MONGO_OP_REPLY_CODE, MONGO_OP_UPDATE_CODE,
};
use crate::cortices::utils::parse_u32;
use crate::declarations::errors::{UnumError, UnumResult};

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

pub fn parse_bson_document(buffer: &[u8]) -> UnumResult<(bson::Document, usize)> {
    let (bson_size, _next_buffer) = parse_u32(
        buffer,
        UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
    )?;
    match bson::decode_document(&mut &(*buffer)[0..(bson_size as usize)]) {
        Err(error) => Err(UnumError::MongoParser(MongoParserError::ParseBsonError(
            error,
        ))),
        Ok(bson_document) => Ok((bson_document, bson_size as usize)),
    }
}
