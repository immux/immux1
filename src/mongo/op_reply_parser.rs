use crate::mongo::error::ParseError;
use crate::mongo::format::OpReply;
use crate::mongo::format::MsgHeader;
use crate::mongo::utils::{parse_bson_document, parse_u64, parse_u32};

pub fn parse_op_reply(message_header: MsgHeader, buffer: &[u8]) -> Result<OpReply, ParseError> {
    let (response_flags, next_buffer) = parse_u32(buffer)?;
    let (cursor_id, next_buffer) = parse_u64(next_buffer)?;
    let (starting_from, next_buffer) = parse_u32(next_buffer)?;
    let (number_returned, mut next_buffer) = parse_u32(next_buffer)?;
    let mut documents = vec![];
    /*
    TODO: Here we have an assumption that official mongo client gives us correct input buffer,
    which means number_returned correctly tells us the following buffer contains exactly this much documents
    */
    for _ in 0..number_returned {
        let (document, rest_buffer) = parse_bson_document(next_buffer)?;
        next_buffer = rest_buffer;
        documents.push(document);
    }
//     Extra check here for preventing official mongo client gives us incorrect input buffer
    if next_buffer.len() != 0 {
        return Err(ParseError::InputBufferError);
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
