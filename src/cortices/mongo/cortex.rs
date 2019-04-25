use std::ffi::CStr;
use std::net::TcpStream;

use bson::{Bson, Document};
use chrono::Utc;

use crate::config::{load_config, UnumDBConfiguration};
use crate::cortices::mongo::ops::msg_header::MsgHeader;
use crate::cortices::mongo::ops::op::MongoOp;
use crate::cortices::mongo::ops::op_msg::{serialize_op_msg, OpMsg, OpMsgFlags, Section};
use crate::cortices::mongo::ops::op_reply::{
    serialize_op_reply, serialize_op_reply_response_flags, OpReply, OpReplyResponseFlags,
};
use crate::cortices::mongo::ops::opcodes::MongoOpCode;
use crate::cortices::mongo::parser::parse_mongo_incoming_bytes;
use crate::cortices::mongo::transformer::{transform_answer_for_mongo, transform_mongo_op};
use crate::cortices::Cortex;
use crate::declarations::errors::{UnumError, UnumResult};
use crate::storage::core::{CoreStore, UnumCore};
use crate::utils::{pretty_dump, u32_to_u8_array};

const ADMIN_QUERY: &str = "admin.$cmd";

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

fn construct_single_doc_op_msg(doc: Document, incoming_header: &MsgHeader) -> OpMsg {
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
        checksum: None,
    }
}

// @see https://github.com/immux/immux/issues/37
fn serialize_op_with_computed_length<OP>(
    op: &OP,
    serializer: &Fn(&OP) -> UnumResult<Vec<u8>>,
) -> UnumResult<Vec<u8>> {
    match serializer(op) {
        Err(error) => Err(error),
        Ok(mut vec) => {
            let correct_width = u32_to_u8_array(vec.len() as u32);
            vec[0] = correct_width[0];
            vec[1] = correct_width[1];
            vec[2] = correct_width[2];
            vec[3] = correct_width[3];
            Ok(vec)
        }
    }
}

fn handle_exceptional_query(
    op: &MongoOp,
    core: &mut UnumCore,
    stream: &TcpStream,
    config: &UnumDBConfiguration,
) -> ExceptionQueryHandlerResult {
    match op {
        MongoOp::Query(op_query) => {
            if &op_query.full_collection_name == ADMIN_QUERY {
                if let Ok(config) = load_config(core) {
                    let document = make_bson_from_config(&config);
                    let mut op_reply = OpReply {
                        message_header: MsgHeader {
                            message_length: 0,
                            request_id: 0,
                            response_to: op_query.message_header.request_id,
                            op_code: MongoOpCode::OpReply,
                        },
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
                    match serialize_op_with_computed_length(&op_reply, &serialize_op_reply) {
                        Err(_error) => ExceptionQueryHandlerResult::Exceptional(Err(
                            UnumError::SerializationFail,
                        )),
                        Ok(data) => ExceptionQueryHandlerResult::Exceptional(Ok(Some(data))),
                    }
                } else {
                    return ExceptionQueryHandlerResult::NotExceptional;
                }
            } else {
                return ExceptionQueryHandlerResult::NotExceptional;
            }
        }
        MongoOp::Msg(op_msg) => {
            fn construct_reply_result(
                response_doc: Document,
                incoming_header: &MsgHeader,
            ) -> ExceptionQueryHandlerResult {
                let reply = construct_single_doc_op_msg(response_doc, incoming_header);
                match serialize_op_with_computed_length(&reply, &serialize_op_msg) {
                    Err(_error) => {
                        ExceptionQueryHandlerResult::Exceptional(Err(UnumError::SerializationFail))
                    }
                    Ok(data) => ExceptionQueryHandlerResult::Exceptional(Ok(Some(data))),
                }
            }

            fn construct_build_info(config: &UnumDBConfiguration) -> Document {
                let mut response_doc = Document::new();
                response_doc.insert("version", "4.0.1");
                response_doc.insert("gitVersion", "");
                response_doc.insert("modules", vec![]);
                response_doc.insert("allocator", "system");
                response_doc.insert("javascriptEngine", "mozjs");
                response_doc.insert("sysInfo", "deprecated");
                response_doc.insert(
                    "versionArray",
                    vec![
                        Bson::I32(4i32),
                        Bson::I32(0i32),
                        Bson::I32(1i32),
                        Bson::I32(0i32),
                    ],
                );

                let mut openssl_doc = Document::new();
                openssl_doc.insert("running", "Apple Secure Transport");
                response_doc.insert("openssl", openssl_doc);

                let mut build_environment_doc = Document::new();
                build_environment_doc.insert("distmod", "");
                build_environment_doc.insert("distarch", "x86_64");
                build_environment_doc.insert("cc", "");
                build_environment_doc.insert("ccflags", "");
                build_environment_doc.insert("cxx", "");
                build_environment_doc.insert("linkflags", "");
                build_environment_doc.insert("target_arch", "x86_64");
                build_environment_doc.insert("target_os", "macOS");
                response_doc.insert("buildEnvironment", build_environment_doc);

                response_doc.insert("bits", 64i32);
                response_doc.insert("debug", false);
                response_doc.insert("maxBsonObjectSize", config.max_bson_object_size);
                response_doc.insert(
                    "storageEngines",
                    vec![
                        Bson::String("hashmap".to_string()),
                        Bson::String("redis".to_string()),
                        Bson::String("rocks".to_string()),
                    ],
                );
                response_doc.insert("ok", 1.0);
                response_doc
            }

            let first_section = &op_msg.sections[0];
            match first_section {
                Section::Single(request_doc) => {
                    if let Ok(1) = request_doc.get_i32("whatsmyuri") {
                        let mut response_doc = Document::new();
                        response_doc.insert("ok", 1);
                        match stream.peer_addr() {
                            Err(error) => {
                                return ExceptionQueryHandlerResult::Exceptional(Err(
                                    UnumError::ReadError,
                                ));
                            }
                            Ok(addr) => {
                                response_doc.insert("you", addr.to_string());
                            }
                        }
                        construct_reply_result(response_doc, &op_msg.message_header)
                    } else if let Ok(1.0) = request_doc.get_f64("buildinfo") {
                        let response_doc = construct_build_info(config);
                        construct_reply_result(response_doc, &op_msg.message_header)
                    } else if let Ok(1.0) = request_doc.get_f64("buildInfo") {
                        let response_doc = construct_build_info(config);
                        construct_reply_result(response_doc, &op_msg.message_header)
                    } else if let Ok(target) = request_doc.get_str("getLog") {
                        let mut response_doc = Document::new();
                        response_doc.insert("totalLinesWritten", 0i32);
                        response_doc.insert("log", vec![]);
                        response_doc.insert("ok", 1.0);
                        construct_reply_result(response_doc, &op_msg.message_header)
                    } else if let Ok(1.0) = request_doc.get_f64("getFreeMonitoringStatus") {
                        let mut response_doc = Document::new();
                        response_doc.insert("state", "undecided");
                        response_doc.insert("ok", 1.0);
                        construct_reply_result(response_doc, &op_msg.message_header)
                    } else if let Ok(1.0) = request_doc.get_f64("isMaster") {
                        let mut response_doc = make_bson_from_config(config);
                        construct_reply_result(response_doc, &op_msg.message_header)
                    } else if let Ok(1.0) = request_doc.get_f64("replSetGetStatus") {
                        let mut response_doc = Document::new();
                        response_doc.insert("ok", 1.0);
                        response_doc.insert("errmsg", "not running with --replSet");
                        response_doc.insert("code", 76i32);
                        response_doc.insert("codeName", "NoReplicationEnabled");
                        construct_reply_result(response_doc, &op_msg.message_header)
                    } else {
                        return ExceptionQueryHandlerResult::NotExceptional;
                    }
                }
                _ => return ExceptionQueryHandlerResult::NotExceptional,
            }
        }
        _ => return ExceptionQueryHandlerResult::NotExceptional,
    }
}

pub fn mongo_cortex_process_incoming_message(
    bytes: &[u8],
    core: &mut UnumCore,
    stream: &TcpStream,
    config: &UnumDBConfiguration,
) -> UnumResult<Option<Vec<u8>>> {
    let op = parse_mongo_incoming_bytes(bytes)?;
    println!("Incoming op: {:#?}", op);
    match handle_exceptional_query(&op, core, &stream, config) {
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
