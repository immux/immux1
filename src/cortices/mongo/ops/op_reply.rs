use bson::Document;

use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::mongo::ops::msg_header::{serialize_msg_header, MsgHeader};
use crate::cortices::mongo::utils::parse_bson_document;
use crate::cortices::utils::{parse_u32, parse_u64};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::utils::{get_bit_u32, set_bit_u32, u32_to_u8_array, u64_to_u8_array};

/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-reply

pub struct OpReplyResponseFlags {
    pub cursor_not_found: bool,
    pub query_failure: bool,
    pub shard_config_stale: bool,
    pub await_capable: bool,
}

#[allow(dead_code)]
fn parse_op_reply_response_flags(flag_int: u32) -> OpReplyResponseFlags {
    return OpReplyResponseFlags {
        cursor_not_found: get_bit_u32(flag_int, 0),
        query_failure: get_bit_u32(flag_int, 1),
        shard_config_stale: get_bit_u32(flag_int, 2),
        await_capable: get_bit_u32(flag_int, 3),
    };
}

pub fn serialize_op_reply_response_flags(flags_struct: &OpReplyResponseFlags) -> u32 {
    let mut result: u32 = 0;
    set_bit_u32(&mut result, 0, flags_struct.cursor_not_found);
    set_bit_u32(&mut result, 1, flags_struct.query_failure);
    set_bit_u32(&mut result, 2, flags_struct.shard_config_stale);
    set_bit_u32(&mut result, 3, flags_struct.await_capable);
    return result;
}

#[derive(Debug)]
pub struct OpReply {
    // standard message header
    pub message_header: MsgHeader,

    // bit vector
    pub response_flags: u32,

    // cursor id if client needs to do get more's
    pub cursor_id: u64,

    // where in the cursor this reply is starting
    pub starting_from: u32,

    // number of documents in the reply
    pub number_returned: u32,

    // documents
    pub documents: Vec<Document>,
}

pub fn parse_op_reply(message_header: MsgHeader, buffer: &[u8]) -> ImmuxResult<OpReply> {
    let mut index: usize = 0;
    let (response_flags, offset) = parse_u32(&buffer[index..])?;
    index += offset;
    let (cursor_id, offset) = parse_u64(&buffer[index..])?;
    index += offset;
    let (starting_from, offset) = parse_u32(&buffer[index..])?;
    index += offset;
    let (number_returned, offset) = parse_u32(&buffer[index..])?;
    index += offset;

    let mut documents = vec![];
    for _ in 0..number_returned {
        let (document, offset) = parse_bson_document(&buffer[index..])?;
        index += offset;
        documents.push(document);
    }
    if index != buffer.len() {
        return Err(ImmuxError::MongoParser(MongoParserError::InputBufferError));
    }
    Ok(OpReply {
        message_header,
        response_flags,
        cursor_id,
        starting_from,
        number_returned,
        documents,
    })
}

pub fn serialize_op_reply(op_reply: &OpReply) -> ImmuxResult<Vec<u8>> {
    if (op_reply.number_returned as usize) != op_reply.documents.len() {
        return Err(ImmuxError::MongoSerializer(
            MongoSerializeError::InputObjectError,
        ));
    }
    let mut res_buffer = serialize_msg_header(&op_reply.message_header);
    res_buffer.append(&mut u32_to_u8_array(op_reply.response_flags).to_vec());
    res_buffer.append(&mut u64_to_u8_array(op_reply.cursor_id).to_vec());
    res_buffer.append(&mut u32_to_u8_array(op_reply.starting_from).to_vec());
    res_buffer.append(&mut u32_to_u8_array(op_reply.number_returned).to_vec());
    for document in &op_reply.documents {
        match bson::encode_document(&mut res_buffer, document) {
            Ok(_) => {}
            Err(error) => {
                return Err(ImmuxError::MongoSerializer(
                    MongoSerializeError::SerializeBsonError(error),
                ));
            }
        }
    }
    Ok(res_buffer)
}

#[cfg(test)]
mod op_reply_tests {

    use crate::cortices::mongo::ops::msg_header::parse_msg_header;
    use crate::cortices::mongo::ops::op_reply::{parse_op_reply, serialize_op_reply};
    use crate::cortices::mongo::ops::op_reply::{
        parse_op_reply_response_flags, serialize_op_reply_response_flags, OpReplyResponseFlags,
    };

    #[test]
    fn test_parse_op_reply_response_flags() {
        {
            let flags_0 = parse_op_reply_response_flags(0);
            assert_eq!(flags_0.cursor_not_found, false);
            assert_eq!(flags_0.query_failure, false);
            assert_eq!(flags_0.shard_config_stale, false);
            assert_eq!(flags_0.await_capable, false);
        }
        {
            let flags_1 = parse_op_reply_response_flags(1);
            assert_eq!(flags_1.cursor_not_found, true);
            assert_eq!(flags_1.query_failure, false);
            assert_eq!(flags_1.shard_config_stale, false);
            assert_eq!(flags_1.await_capable, false);
        }
        {
            let flags_2 = parse_op_reply_response_flags(2);
            assert_eq!(flags_2.cursor_not_found, false);
            assert_eq!(flags_2.query_failure, true);
            assert_eq!(flags_2.shard_config_stale, false);
            assert_eq!(flags_2.await_capable, false);
        }
        {
            let flags_4 = parse_op_reply_response_flags(4);
            assert_eq!(flags_4.cursor_not_found, false);
            assert_eq!(flags_4.query_failure, false);
            assert_eq!(flags_4.shard_config_stale, true);
            assert_eq!(flags_4.await_capable, false);
        }
        {
            let flags_8 = parse_op_reply_response_flags(8);
            assert_eq!(flags_8.cursor_not_found, false);
            assert_eq!(flags_8.query_failure, false);
            assert_eq!(flags_8.shard_config_stale, false);
            assert_eq!(flags_8.await_capable, true);
        }
        {
            let flags_15 = parse_op_reply_response_flags(255);
            assert_eq!(flags_15.cursor_not_found, true);
            assert_eq!(flags_15.query_failure, true);
            assert_eq!(flags_15.shard_config_stale, true);
            assert_eq!(flags_15.await_capable, true);
        }
    }

    #[test]
    fn test_serialize_op_reply_response_flags() {
        {
            let flags_0 = serialize_op_reply_response_flags(&OpReplyResponseFlags {
                cursor_not_found: false,
                query_failure: false,
                shard_config_stale: false,
                await_capable: false,
            });
            assert_eq!(flags_0, 0);
        }
        {
            let flags_1 = serialize_op_reply_response_flags(&OpReplyResponseFlags {
                cursor_not_found: true,
                query_failure: false,
                shard_config_stale: false,
                await_capable: false,
            });
            assert_eq!(flags_1, 1);
        }
        {
            let flags_2 = serialize_op_reply_response_flags(&OpReplyResponseFlags {
                cursor_not_found: false,
                query_failure: true,
                shard_config_stale: false,
                await_capable: false,
            });
            assert_eq!(flags_2, 2);
        }
        {
            let flags_4 = serialize_op_reply_response_flags(&OpReplyResponseFlags {
                cursor_not_found: false,
                query_failure: false,
                shard_config_stale: true,
                await_capable: false,
            });
            assert_eq!(flags_4, 4);
        }
        {
            let flags_8 = serialize_op_reply_response_flags(&OpReplyResponseFlags {
                cursor_not_found: false,
                query_failure: false,
                shard_config_stale: false,
                await_capable: true,
            });
            assert_eq!(flags_8, 8);
        }
        {
            let flags_15 = serialize_op_reply_response_flags(&OpReplyResponseFlags {
                cursor_not_found: true,
                query_failure: true,
                shard_config_stale: true,
                await_capable: true,
            });
            assert_eq!(flags_15, 15);
        }
    }

    static OP_REPLY_FIXTURE: [u8; 442] = [
        0xaa, 0x01, 0x00, 0x00, 0x91, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0xcb, 0x00, 0x00, 0x00, 0x08, 0x69, 0x73, 0x6d, 0x61,
        0x73, 0x74, 0x65, 0x72, 0x00, 0x01, 0x10, 0x6d, 0x61, 0x78, 0x42, 0x73, 0x6f, 0x6e, 0x4f,
        0x62, 0x6a, 0x65, 0x63, 0x74, 0x53, 0x69, 0x7a, 0x65, 0x00, 0x00, 0x00, 0x00, 0x01, 0x10,
        0x6d, 0x61, 0x78, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x53, 0x69, 0x7a, 0x65, 0x42,
        0x79, 0x74, 0x65, 0x73, 0x00, 0x00, 0x6c, 0xdc, 0x02, 0x10, 0x6d, 0x61, 0x78, 0x57, 0x72,
        0x69, 0x74, 0x65, 0x42, 0x61, 0x74, 0x63, 0x68, 0x53, 0x69, 0x7a, 0x65, 0x00, 0xa0, 0x86,
        0x01, 0x00, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x54, 0x69, 0x6d, 0x65, 0x00, 0x9e, 0xd1,
        0xfe, 0xbc, 0x69, 0x01, 0x00, 0x00, 0x10, 0x6c, 0x6f, 0x67, 0x69, 0x63, 0x61, 0x6c, 0x53,
        0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x54, 0x69, 0x6d, 0x65, 0x6f, 0x75, 0x74, 0x4d, 0x69,
        0x6e, 0x75, 0x74, 0x65, 0x73, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x10, 0x6d, 0x69, 0x6e, 0x57,
        0x69, 0x72, 0x65, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x10, 0x6d, 0x61, 0x78, 0x57, 0x69, 0x72, 0x65, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e,
        0x00, 0x07, 0x00, 0x00, 0x00, 0x08, 0x72, 0x65, 0x61, 0x64, 0x4f, 0x6e, 0x6c, 0x79, 0x00,
        0x00, 0x01, 0x6f, 0x6b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, 0x00, 0xcb,
        0x00, 0x00, 0x00, 0x08, 0x69, 0x73, 0x6d, 0x61, 0x73, 0x74, 0x65, 0x72, 0x00, 0x01, 0x10,
        0x6d, 0x61, 0x78, 0x42, 0x73, 0x6f, 0x6e, 0x4f, 0x62, 0x6a, 0x65, 0x63, 0x74, 0x53, 0x69,
        0x7a, 0x65, 0x00, 0x00, 0x00, 0x00, 0x01, 0x10, 0x6d, 0x61, 0x78, 0x4d, 0x65, 0x73, 0x73,
        0x61, 0x67, 0x65, 0x53, 0x69, 0x7a, 0x65, 0x42, 0x79, 0x74, 0x65, 0x73, 0x00, 0x00, 0x6c,
        0xdc, 0x02, 0x10, 0x6d, 0x61, 0x78, 0x57, 0x72, 0x69, 0x74, 0x65, 0x42, 0x61, 0x74, 0x63,
        0x68, 0x53, 0x69, 0x7a, 0x65, 0x00, 0xa0, 0x86, 0x01, 0x00, 0x09, 0x6c, 0x6f, 0x63, 0x61,
        0x6c, 0x54, 0x69, 0x6d, 0x65, 0x00, 0x9e, 0xd1, 0xfe, 0xbc, 0x69, 0x01, 0x00, 0x00, 0x10,
        0x6c, 0x6f, 0x67, 0x69, 0x63, 0x61, 0x6c, 0x53, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x54,
        0x69, 0x6d, 0x65, 0x6f, 0x75, 0x74, 0x4d, 0x69, 0x6e, 0x75, 0x74, 0x65, 0x73, 0x00, 0x1e,
        0x00, 0x00, 0x00, 0x10, 0x6d, 0x69, 0x6e, 0x57, 0x69, 0x72, 0x65, 0x56, 0x65, 0x72, 0x73,
        0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x6d, 0x61, 0x78, 0x57, 0x69, 0x72,
        0x65, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x07, 0x00, 0x00, 0x00, 0x08, 0x72,
        0x65, 0x61, 0x64, 0x4f, 0x6e, 0x6c, 0x79, 0x00, 0x00, 0x01, 0x6f, 0x6b, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, 0x00,
    ];

    #[test]
    fn test_parse_op_reply() {
        let buffer = OP_REPLY_FIXTURE;
        let mut index: usize = 0;
        let (header, offset) = parse_msg_header(&buffer[index..]).unwrap();
        index += offset;
        let op_query = parse_op_reply(header, &buffer[index..]).unwrap();
        assert_eq!(op_query.response_flags, 8);
        assert_eq!(op_query.cursor_id, 0);
        assert_eq!(op_query.starting_from, 0);
        assert_eq!(op_query.number_returned, 2);
        assert!(op_query.documents[0].contains_key("ismaster"));
        assert_eq!(
            op_query.documents[0].contains_key("maxBsonObjectSize"),
            true
        );
        assert_eq!(
            op_query.documents[0].contains_key("maxMessageSizeBytes"),
            true
        );
        assert_eq!(
            op_query.documents[0].contains_key("maxWriteBatchSize"),
            true
        );
        assert!(op_query.documents[0].contains_key("localTime"));
        assert_eq!(
            op_query.documents[0].contains_key("logicalSessionTimeoutMinutes"),
            true
        );
        assert!(op_query.documents[0].contains_key("minWireVersion"));
        assert!(op_query.documents[0].contains_key("maxWireVersion"));
        assert!(op_query.documents[0].contains_key("readOnly"));
        assert!(op_query.documents[0].contains_key("ok"));
        assert_eq!(op_query.documents[0].get_f64("ok").unwrap(), 1.0);
        assert_eq!(op_query.documents[0].get_bool("readOnly").unwrap(), false);
    }

    #[test]
    fn test_serialize_op_reply() {
        let buffer = OP_REPLY_FIXTURE;
        let mut index: usize = 0;
        let (header, offset) = parse_msg_header(&buffer[index..]).unwrap();
        index += offset;
        let op_query = parse_op_reply(header, &buffer[index..]).unwrap();
        let op_reply_buffer = serialize_op_reply(&op_query).unwrap();
        assert_eq!(op_reply_buffer, buffer.to_vec());
    }
}
