use crate::mongo::constants::*;
use crate::mongo::format::OpReply;
use crate::mongo::header_serializer::{serialize_msg_header};
use crate::utils::{u32_to_u8_array, u64_to_u8_array};

extern crate bson;

pub fn serialize_op_reply(op_reply: &OpReply) -> Vec<u8> {
    let mut res_buffer = serialize_msg_header(&op_reply.message_header);
    res_buffer.append(&mut u32_to_u8_array(op_reply.response_flags).to_vec());
    res_buffer.append(&mut u64_to_u8_array(op_reply.cursor_id).to_vec());
    res_buffer.append(&mut u32_to_u8_array(op_reply.starting_from).to_vec());
    res_buffer.append(&mut u32_to_u8_array(op_reply.number_returned).to_vec());

    for _ in 0..op_reply.number_returned {
        bson::encode_document(&mut res_buffer, &op_reply.documents[0]);
    }

    res_buffer
}
