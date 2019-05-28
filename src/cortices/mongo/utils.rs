use bson::{Document, ValueAccessResult};
use chrono::Utc;

use crate::config::ImmuxDBConfiguration;
use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::msg_header::MsgHeader;
use crate::cortices::mongo::ops::op_msg::{OpMsg, OpMsgFlags, Section};
use crate::cortices::mongo::ops::opcodes::{
    MongoOpCode, MONGO_OP_COMMAND_CODE, MONGO_OP_COMMAND_REPLY_CODE, MONGO_OP_DELETE_CODE,
    MONGO_OP_GET_MORE_CODE, MONGO_OP_INSERT_CODE, MONGO_OP_KILL_CURSORS_CODE, MONGO_OP_MSG_CODE,
    MONGO_OP_QUERY_CODE, MONGO_OP_REPLY_CODE, MONGO_OP_UPDATE_CODE,
};
use crate::cortices::utils::parse_u32;
use crate::declarations::errors::{ImmuxError, ImmuxResult};

pub fn pick_op_code(op: u32) -> ImmuxResult<MongoOpCode> {
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
        _ => Err(ImmuxError::MongoParser(MongoParserError::UnknownOpCode(op))),
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

pub fn parse_bson_document(buffer: &[u8]) -> ImmuxResult<(bson::Document, usize)> {
    let (bson_size, _next_buffer) = parse_u32(buffer)?;
    match bson::decode_document(&mut &(*buffer)[0..(bson_size as usize)]) {
        Err(error) => Err(ImmuxError::MongoParser(MongoParserError::ParseBsonError(
            error,
        ))),
        Ok(bson_document) => Ok((bson_document, bson_size as usize)),
    }
}

pub fn construct_single_doc_op_msg(doc: Document, incoming_header: &MsgHeader) -> OpMsg {
    OpMsg {
        message_header: MsgHeader {
            message_length: 0,
            request_id: 0,
            response_to: incoming_header.request_id,
            op_code: MongoOpCode::OpMsg,
        },
        flags: OpMsgFlags {
            check_sum_present: false,
            more_to_come: false,
            exhaust_allowed: false,
        },
        sections: vec![Section::Single(doc)],
    }
}

pub fn make_bson_from_config(config: &ImmuxDBConfiguration) -> bson::Document {
    let mut document = bson::Document::new();
    document.insert("ismaster", config.is_master);
    document.insert("maxBsonObjectSize", config.max_bson_object_size as i32);
    document.insert(
        "maxMessageSizeBytes",
        config.max_message_size_in_bytes as i32,
    );
    document.insert("maxWriteBatchSize", config.max_write_batch_size as i32);
    document.insert("localTime", Utc::now());
    document.insert(
        "logicalSessionTimeoutMinutes",
        config.logical_session_timeout_minutes,
    );
    document.insert("minWireVersion", config.min_mongo_wire_version);
    document.insert("maxWireVersion", config.max_mongo_wire_version);
    document.insert("readOnly", config.read_only);
    document.insert("ok", 1.0);
    return document;
}

pub fn is_1(parsed: ValueAccessResult<f64>) -> bool {
    match parsed {
        Ok(number) => number == 1.0,
        _ => false,
    }
}
