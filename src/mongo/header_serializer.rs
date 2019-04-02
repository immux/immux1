use crate::mongo::constants::*;
use crate::mongo::format::MsgHeader;
use crate::utils::{u32_to_u8_array};

pub fn serialize_msg_header(message_header: &MsgHeader) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    res.append(&mut u32_to_u8_array(message_header.message_length).to_vec());
    res.append(&mut u32_to_u8_array(message_header.request_id).to_vec());
    res.append(&mut u32_to_u8_array(message_header.response_to).to_vec());
    res.append(&mut u32_to_u8_array(message_header.op_code).to_vec());
    res
}
