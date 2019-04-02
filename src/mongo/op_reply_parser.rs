use crate::mongo::error::ParseError;
use crate::mongo::format::OpReply;
use crate::mongo::format::MsgHeader;
use crate::mongo::utils::{parse_bson_document, parse_u64, parse_u32};

pub fn parse_op_reply(message_header: MsgHeader, buffer: &[u8]) -> Result<OpReply, ParseError> {
    let (response_flags, next_buffer) = parse_u32(buffer)?;
    let (cursor_id, next_buffer) = parse_u64(next_buffer)?;
    let (starting_from, next_buffer) = parse_u32(next_buffer)?;
    let (number_returned, next_buffer) = parse_u32(next_buffer)?;
    let mut documents = vec![];
    for _ in 0..number_returned {
        let (document, next_buffer) = parse_bson_document(next_buffer).unwrap();
        documents.push(document);
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
