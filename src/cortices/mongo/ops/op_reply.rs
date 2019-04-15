use bson::Document;

use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::mongo::ops::msg_header::{serialize_msg_header, MsgHeader};
use crate::cortices::mongo::utils::{parse_bson_document, parse_u32, parse_u64};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::{get_bit_u32, set_bit_u32, u32_to_u8_array, u64_to_u8_array};

/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-reply

pub struct OpReplyResponseFlags {
    pub cursor_not_found: bool,
    pub query_failure: bool,
    pub shard_config_stale: bool,
    pub await_capable: bool,
}

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

pub fn parse_op_reply(message_header: MsgHeader, buffer: &[u8]) -> UnumResult<OpReply> {
    let (response_flags, next_buffer) = parse_u32(buffer)?;
    let (cursor_id, next_buffer) = parse_u64(next_buffer)?;
    let (starting_from, next_buffer) = parse_u32(next_buffer)?;
    let (number_returned, mut next_buffer) = parse_u32(next_buffer)?;
    let mut documents = vec![];
    // TODO(#32)
    for _ in 0..number_returned {
        let (document, rest_buffer) = parse_bson_document(next_buffer)?;
        next_buffer = rest_buffer;
        documents.push(document);
    }
    if next_buffer.len() != 0 {
        return Err(UnumError::MongoParser(MongoParserError::InputBufferError));
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

pub fn serialize_op_reply(op_reply: &OpReply) -> UnumResult<Vec<u8>> {
    if (op_reply.number_returned as usize) != op_reply.documents.len() {
        return Err(UnumError::MongoSerializer(
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
                return Err(UnumError::MongoSerializer(
                    MongoSerializeError::SerializeBsonError(error),
                ));
            }
        }
    }
    Ok(res_buffer)
}

#[cfg(test)]
mod tests {
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
}
