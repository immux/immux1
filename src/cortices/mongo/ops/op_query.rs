use bson::Document;

use crate::cortices::mongo::ops::msg_header::{parse_msg_header, MsgHeader};
use crate::cortices::mongo::utils::parse_bson_document;
use crate::cortices::utils::{parse_cstring, parse_u32};
use crate::declarations::errors::UnumResult;

/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-query
#[derive(Debug)]
pub struct OpQuery {
    // standard message header
    pub message_header: MsgHeader,

    // bit vector of query options.
    pub flags: u32,

    // "dbname.collectionname"
    pub full_collection_name: String,

    // number of documents to skip
    pub number_to_skip: u32,

    // number of documents to return in the first OP_REPLY batch
    pub number_to_return: u32,

    // query object.
    pub query: Document,

    // Optional. Selector indicating the fields to return.
    pub return_fields_selector: Option<Document>,
}

pub fn parse_op_query(buffer: &[u8]) -> UnumResult<OpQuery> {
    let mut index: usize = 0;
    let (message_header, offset) = parse_msg_header(&buffer[index..])?;
    index += offset;
    let (flags, offset) = parse_u32(&buffer[index..])?;
    index += offset;
    let (full_collection_name, offset) = parse_cstring(&buffer[index..])?;
    index += offset;
    let (number_to_skip, offset) = parse_u32(&buffer[index..])?;
    index += offset;
    let (number_to_return, offset) = parse_u32(&buffer[index..])?;
    index += offset;
    let (query, offset) = parse_bson_document(&buffer[index..])?;
    index += offset;
    let return_fields_selector = if index == buffer.len() {
        None
    } else {
        Some(parse_bson_document(&buffer[index..])?.0)
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

#[cfg(test)]
mod op_query_tests {

    use crate::cortices::mongo::ops::msg_header::parse_msg_header;
    use crate::cortices::mongo::ops::op_query::parse_op_query;
    use crate::cortices::mongo::utils::{get_op_code_value, parse_bson_document};
    use crate::cortices::utils::{parse_cstring, parse_u32};

    static OP_QUERY_FIXTURE: [u8; 269] = [
        0x0d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd4, 0x07, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x61, 0x64, 0x6d, 0x69, 0x6e, 0x2e, 0x24, 0x63, 0x6d, 0x64,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xe6, 0x00, 0x00, 0x00, 0x10, 0x69,
        0x73, 0x4d, 0x61, 0x73, 0x74, 0x65, 0x72, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x63, 0x6c,
        0x69, 0x65, 0x6e, 0x74, 0x00, 0xcb, 0x00, 0x00, 0x00, 0x03, 0x61, 0x70, 0x70, 0x6c, 0x69,
        0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x00, 0x1d, 0x00, 0x00, 0x00, 0x02, 0x6e, 0x61, 0x6d,
        0x65, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x4d, 0x6f, 0x6e, 0x67, 0x6f, 0x44, 0x42, 0x20, 0x53,
        0x68, 0x65, 0x6c, 0x6c, 0x00, 0x00, 0x03, 0x64, 0x72, 0x69, 0x76, 0x65, 0x72, 0x00, 0x3a,
        0x00, 0x00, 0x00, 0x02, 0x6e, 0x61, 0x6d, 0x65, 0x00, 0x18, 0x00, 0x00, 0x00, 0x4d, 0x6f,
        0x6e, 0x67, 0x6f, 0x44, 0x42, 0x20, 0x49, 0x6e, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x6c, 0x20,
        0x43, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x00, 0x02, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e,
        0x00, 0x06, 0x00, 0x00, 0x00, 0x34, 0x2e, 0x30, 0x2e, 0x31, 0x00, 0x00, 0x03, 0x6f, 0x73,
        0x00, 0x56, 0x00, 0x00, 0x00, 0x02, 0x74, 0x79, 0x70, 0x65, 0x00, 0x07, 0x00, 0x00, 0x00,
        0x44, 0x61, 0x72, 0x77, 0x69, 0x6e, 0x00, 0x02, 0x6e, 0x61, 0x6d, 0x65, 0x00, 0x09, 0x00,
        0x00, 0x00, 0x4d, 0x61, 0x63, 0x20, 0x4f, 0x53, 0x20, 0x58, 0x00, 0x02, 0x61, 0x72, 0x63,
        0x68, 0x69, 0x74, 0x65, 0x63, 0x74, 0x75, 0x72, 0x65, 0x00, 0x07, 0x00, 0x00, 0x00, 0x78,
        0x38, 0x36, 0x5f, 0x36, 0x34, 0x00, 0x02, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00,
        0x07, 0x00, 0x00, 0x00, 0x31, 0x38, 0x2e, 0x32, 0x2e, 0x30, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn test_parse_msg_header() {
        let buffer = OP_QUERY_FIXTURE;
        let (message_header, _) = parse_msg_header(&buffer).unwrap();
        assert_eq!(message_header.message_length, 269);
        assert_eq!(message_header.request_id, 0);
        assert_eq!(message_header.response_to, 0);
        assert_eq!(get_op_code_value(&message_header.op_code), 2004);
    }

    #[test]
    fn test_parse_bson_document() {
        let buffer = OP_QUERY_FIXTURE;
        let mut index: usize = 0;
        let (_, offset) = parse_msg_header(&buffer[index..]).unwrap();
        index += offset;
        let (_, offset) = parse_u32(&buffer[index..]).unwrap();
        index += offset;
        let (_, offset) = parse_cstring(&buffer[index..]).unwrap();
        index += offset;
        let (_, offset) = parse_u32(&buffer[index..]).unwrap();
        index += offset;
        let (_, offset) = parse_u32(&buffer[index..]).unwrap();
        index += offset;
        let (doc, _) = parse_bson_document(&buffer[index..]).unwrap();
        assert!(doc.contains_key("isMaster"));
        assert!(doc.contains_key("client"));
    }

    #[test]
    fn test_parse_op_query() {
        let buffer = OP_QUERY_FIXTURE;
        let mut index: usize = 0;
        let op_query = parse_op_query(&buffer[index..]).unwrap();
        assert_eq!(op_query.flags, 0);
        assert_eq!(op_query.number_to_skip, 0);
        assert_eq!(op_query.number_to_return, 1);
        assert_eq!(op_query.return_fields_selector, None);
    }
}
