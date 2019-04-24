use std::ffi::CStr;

use chrono::Utc;

use crate::config::{load_config, UnumDBConfiguration};
use crate::cortices::mongo::ops::msg_header::MsgHeader;
use crate::cortices::mongo::ops::op::MongoOp;
use crate::cortices::mongo::ops::op_reply::{
    serialize_op_reply, serialize_op_reply_response_flags, OpReply, OpReplyResponseFlags,
};
use crate::cortices::mongo::ops::opcodes::MongoOpCode;
use crate::cortices::mongo::parser::parse_mongo_incoming_bytes;
use crate::cortices::mongo::transformer::{transform_answer_for_mongo, transform_mongo_op};
use crate::cortices::Cortex;
use crate::declarations::errors::{UnumError, UnumResult};
use crate::storage::core::{CoreStore, UnumCore};
use crate::utils::pretty_dump;

const ADMIN_QUERY: &str = "admin.$cmd";

fn cstr_eq_str(cstr: &CStr, s: &str) -> bool {
    match cstr.to_str() {
        Err(_) => return false,
        Ok(c) => return c == s,
    }
}

fn make_bson_from_config(config: &UnumDBConfiguration) -> bson::Document {
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

enum ExceptionQueryHandlerResult {
    NotExceptional,
    Exceptional(UnumResult<Option<Vec<u8>>>),
}

fn handle_exceptional_query(op: &MongoOp, core: &mut UnumCore) -> ExceptionQueryHandlerResult {
    match op {
        MongoOp::Query(op_query) => {
            if cstr_eq_str(&op_query.full_collection_name, ADMIN_QUERY) {
                if let Ok(config) = load_config(core) {
                    let document = make_bson_from_config(&config);
                    let mut header = MsgHeader {
                        message_length: 0,
                        request_id: 0,
                        response_to: op_query.message_header.request_id,
                        op_code: MongoOpCode::OpReply,
                    };
                    let mut op_reply = OpReply {
                        message_header: header.clone(),
                        response_flags: serialize_op_reply_response_flags(&OpReplyResponseFlags {
                            cursor_not_found: false,
                            query_failure: false,
                            shard_config_stale: false,
                            await_capable: false,
                        }),
                        cursor_id: 0,
                        starting_from: 0,
                        number_returned: 1,
                        documents: vec![document],
                    };
                    match serialize_op_reply(&op_reply) {
                        Err(_error) => {
                            return ExceptionQueryHandlerResult::Exceptional(Err(
                                UnumError::SerializationFail,
                            ));
                        }
                        Ok(vec) => {
                            header.message_length = vec.len() as u32;
                            op_reply.message_header = header.clone();
                            // Serialize twice to get the length right. See issue #37.
                            match serialize_op_reply(&op_reply) {
                                Err(_error) => {
                                    return ExceptionQueryHandlerResult::Exceptional(Err(
                                        UnumError::SerializationFail,
                                    ));
                                }
                                Ok(actual_data) => {
                                    return ExceptionQueryHandlerResult::Exceptional(Ok(Some(
                                        actual_data,
                                    )));
                                }
                            }
                        }
                    };
                } else {
                    return ExceptionQueryHandlerResult::NotExceptional;
                }
            } else {
                return ExceptionQueryHandlerResult::NotExceptional;
            }
        }
        _ => return ExceptionQueryHandlerResult::NotExceptional,
    }
}

pub fn mongo_cortex_process_incoming_message(
    bytes: &[u8],
    core: &mut UnumCore,
) -> UnumResult<Option<Vec<u8>>> {
    pretty_dump(bytes);
    let op = parse_mongo_incoming_bytes(bytes)?;
    println!("Incoming op: {:#?}", op);
    match handle_exceptional_query(&op, core) {
        ExceptionQueryHandlerResult::Exceptional(result) => return result,
        ExceptionQueryHandlerResult::NotExceptional => {
            let instruction = transform_mongo_op(&op)?;
            let answer = core.execute(&instruction)?;
            let op_reply = transform_answer_for_mongo(&answer)?;
            match serialize_op_reply(&op_reply) {
                Err(error) => return Err(error),
                Ok(reply_bytes) => {
                    return Ok(Some(reply_bytes));
                }
            }
        }
    };
}

pub const MONGO_CORTEX: Cortex = Cortex {
    process_incoming_message: mongo_cortex_process_incoming_message,
    process_first_connection: None,
};
