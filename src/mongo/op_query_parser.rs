use crate::mongo::constants::*;
use crate::mongo::error::ParseError;
use crate::mongo::format::MsgHeader;
use crate::mongo::format::OpQuery;
use crate::mongo::utils::{parse_bson_document, parse_cstring, parse_u32};

extern crate bson;

pub fn parse_op_query(message_header: MsgHeader, buffer: &[u8]) -> Result<OpQuery, ParseError> {
    let (flags, next_buffer) = parse_u32(buffer)?;
    let (full_collection_name, next_buffer) = parse_cstring(next_buffer)?;
    let (number_to_skip, next_buffer) = parse_u32(next_buffer)?;
    let (number_to_return, next_buffer) = parse_u32(next_buffer)?;
    let (query, next_buffer) = parse_bson_document(next_buffer)?;
    let return_fields_selector = if next_buffer.is_empty() {
        None
    } else {
        Some(parse_bson_document(next_buffer)?.0)
    };
    Ok(OpQuery {
        message_header,
        flags,
        full_collection_name,
        number_to_skip,
        number_to_return,
        query,
        return_fields_selector,
    })
}
