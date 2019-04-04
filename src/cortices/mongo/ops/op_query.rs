use std::ffi::CString;

use bson::Document;

use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::msg_header::MsgHeader;
use crate::cortices::mongo::utils::{parse_bson_document, parse_cstring, parse_u32};

/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-query
#[derive(Debug)]
pub struct OpQuery {
    // standard message header
    pub message_header: MsgHeader,

    // bit vector of query options.
    pub flags: u32,

    // "dbname.collectionname"
    pub full_collection_name: CString,

    // number of documents to skip
    pub number_to_skip: u32,

    // number of documents to return in the first OP_REPLY batch
    pub number_to_return: u32,

    // query object.
    pub query: Document,

    // Optional. Selector indicating the fields to return.
    pub return_fields_selector: Option<Document>,
}

pub fn parse_op_query(
    message_header: MsgHeader,
    buffer: &[u8],
) -> Result<OpQuery, MongoParserError> {
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
