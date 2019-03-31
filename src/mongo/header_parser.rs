use crate::mongo::constants::*;
use crate::mongo::error::ParseError;
use crate::mongo::format::MsgHeader;
use crate::mongo::utils::parse_u32;

pub fn parse_msg_header(buffer: &[u8]) -> Result<(MsgHeader, &[u8]), ParseError> {
    let (message_length, next_buffer) = parse_u32(buffer)?;
    let (request_id, next_buffer) = parse_u32(next_buffer)?;
    let (response_to, next_buffer) = parse_u32(next_buffer)?;
    let (op_code, next_buffer) = parse_u32(next_buffer)?;
    Ok((
        MsgHeader {
            message_length,
            request_id,
            response_to,
            op_code,
        },
        next_buffer,
    ))
}
