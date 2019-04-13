use std::ffi::CString;

use bson::Document;

use crate::cortices::mongo::ops::msg_header::MsgHeader;
use crate::cortices::mongo::utils::{parse_bson_document, parse_cstring, parse_u32};
use crate::declarations::errors::UnumResult;

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

pub fn parse_op_query(message_header: MsgHeader, buffer: &[u8]) -> UnumResult<OpQuery> {
    let (flags, next_buffer) = parse_u32(buffer)?;
    let (full_collection_name, _cstring_size, next_buffer) = parse_cstring(next_buffer)?;
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

#[cfg(test)]
mod op_query_tests {

    use crate::cortices::mongo::ops::msg_header::parse_msg_header;
    use crate::cortices::mongo::ops::op_query::parse_op_query;
    use crate::cortices::mongo::utils::{
        get_op_code_value, parse_bson_document, parse_cstring, parse_u32,
    };

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
    fn test_parse_cstring() {
        let buffer = OP_QUERY_FIXTURE;
        let (_, next_buffer) = parse_msg_header(&buffer).unwrap();
        let (_, next_buffer) = parse_u32(next_buffer).unwrap();
        let (res, _, _) = parse_cstring(next_buffer).unwrap();
        assert_eq!(res.to_str().unwrap(), "admin.$cmd");
    }

    #[test]
    fn test_parse_cstring_error() {
        let buffer = [0x70, 0x70, 0x6c, 0x69];
        match parse_cstring(&buffer) {
            Ok((_, _, _)) => {
                assert!(false, "buffer is not a legal format for cstring");
            }
            Err(_) => {
                assert!(true);
            }
        }
    }

    #[test]
    fn test_parse_bson_document() {
        let buffer = OP_QUERY_FIXTURE;
        let (_, next_buffer) = parse_msg_header(&buffer).unwrap();
        let (_, next_buffer) = parse_u32(next_buffer).unwrap();
        let (_, _, next_buffer) = parse_cstring(next_buffer).unwrap();
        let (_, next_buffer) = parse_u32(next_buffer).unwrap();
        let (_, next_buffer) = parse_u32(next_buffer).unwrap();
        let (doc, _) = parse_bson_document(next_buffer).unwrap();
        assert_eq!(doc.contains_key("isMaster"), true);
        assert_eq!(doc.contains_key("client"), true);
    }

    #[test]
    fn test_parse_op_query() {
        let buffer = OP_QUERY_FIXTURE;
        let (header, next_buffer) = parse_msg_header(&buffer).unwrap();
        let op_query = parse_op_query(header, next_buffer).unwrap();
        assert_eq!(op_query.flags, 0);
        assert_eq!(op_query.number_to_skip, 0);
        assert_eq!(op_query.number_to_return, 1);
        assert_eq!(op_query.return_fields_selector, None);
    }
}
