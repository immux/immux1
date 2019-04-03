#[cfg(test)]
mod tests {
    use crate::cortices::mongo::ops::msg_header::parse_msg_header;
    use crate::cortices::mongo::ops::op_reply::{parse_op_reply, serialize_op_reply};
    use crate::cortices::mongo::ops::op_query::{parse_op_query};
    use crate::cortices::mongo::utils::{parse_u32, parse_u64, parse_cstring, parse_bson_document, get_op_code_value};

    static OP_REPLY_BUFFER: [u8; 442] = [
        0xaa,0x01,0x00,0x00,0x91,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01,0x00,0x00,0x00,
        0x08,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x02,0x00,0x00,0x00,0xcb,0x00,0x00,0x00,0x08,0x69,0x73,0x6d,0x61,0x73,0x74,0x65,
        0x72,0x00,0x01,0x10,0x6d,0x61,0x78,0x42,0x73,0x6f,0x6e,0x4f,0x62,0x6a,0x65,0x63,
        0x74,0x53,0x69,0x7a,0x65,0x00,0x00,0x00,0x00,0x01,0x10,0x6d,0x61,0x78,0x4d,0x65,
        0x73,0x73,0x61,0x67,0x65,0x53,0x69,0x7a,0x65,0x42,0x79,0x74,0x65,0x73,0x00,0x00,
        0x6c,0xdc,0x02,0x10,0x6d,0x61,0x78,0x57,0x72,0x69,0x74,0x65,0x42,0x61,0x74,0x63,
        0x68,0x53,0x69,0x7a,0x65,0x00,0xa0,0x86,0x01,0x00,0x09,0x6c,0x6f,0x63,0x61,0x6c,
        0x54,0x69,0x6d,0x65,0x00,0x9e,0xd1,0xfe,0xbc,0x69,0x01,0x00,0x00,0x10,0x6c,0x6f,
        0x67,0x69,0x63,0x61,0x6c,0x53,0x65,0x73,0x73,0x69,0x6f,0x6e,0x54,0x69,0x6d,0x65,
        0x6f,0x75,0x74,0x4d,0x69,0x6e,0x75,0x74,0x65,0x73,0x00,0x1e,0x00,0x00,0x00,0x10,
        0x6d,0x69,0x6e,0x57,0x69,0x72,0x65,0x56,0x65,0x72,0x73,0x69,0x6f,0x6e,0x00,0x00,
        0x00,0x00,0x00,0x10,0x6d,0x61,0x78,0x57,0x69,0x72,0x65,0x56,0x65,0x72,0x73,0x69,
        0x6f,0x6e,0x00,0x07,0x00,0x00,0x00,0x08,0x72,0x65,0x61,0x64,0x4f,0x6e,0x6c,0x79,
        0x00,0x00,0x01,0x6f,0x6b,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xf0,0x3f,0x00,0xcb,
        0x00,0x00,0x00,0x08,0x69,0x73,0x6d,0x61,0x73,0x74,0x65,0x72,0x00,0x01,0x10,0x6d,
        0x61,0x78,0x42,0x73,0x6f,0x6e,0x4f,0x62,0x6a,0x65,0x63,0x74,0x53,0x69,0x7a,0x65,
        0x00,0x00,0x00,0x00,0x01,0x10,0x6d,0x61,0x78,0x4d,0x65,0x73,0x73,0x61,0x67,0x65,
        0x53,0x69,0x7a,0x65,0x42,0x79,0x74,0x65,0x73,0x00,0x00,0x6c,0xdc,0x02,0x10,0x6d,
        0x61,0x78,0x57,0x72,0x69,0x74,0x65,0x42,0x61,0x74,0x63,0x68,0x53,0x69,0x7a,0x65,
        0x00,0xa0,0x86,0x01,0x00,0x09,0x6c,0x6f,0x63,0x61,0x6c,0x54,0x69,0x6d,0x65,0x00,
        0x9e,0xd1,0xfe,0xbc,0x69,0x01,0x00,0x00,0x10,0x6c,0x6f,0x67,0x69,0x63,0x61,0x6c,
        0x53,0x65,0x73,0x73,0x69,0x6f,0x6e,0x54,0x69,0x6d,0x65,0x6f,0x75,0x74,0x4d,0x69,
        0x6e,0x75,0x74,0x65,0x73,0x00,0x1e,0x00,0x00,0x00,0x10,0x6d,0x69,0x6e,0x57,0x69,
        0x72,0x65,0x56,0x65,0x72,0x73,0x69,0x6f,0x6e,0x00,0x00,0x00,0x00,0x00,0x10,0x6d,
        0x61,0x78,0x57,0x69,0x72,0x65,0x56,0x65,0x72,0x73,0x69,0x6f,0x6e,0x00,0x07,0x00,
        0x00,0x00,0x08,0x72,0x65,0x61,0x64,0x4f,0x6e,0x6c,0x79,0x00,0x00,0x01,0x6f,0x6b,
        0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xf0,0x3f,0x00
    ];

    #[test]
    fn test_parse_u64() {
        let res = parse_u64(&[0x0d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        let (num, next_buffer) = res.unwrap();
        assert_eq!(269, num);
        assert_eq!(next_buffer.len(), 0);
    }

    #[test]
    fn test_parse_u64_error() {
        match parse_u64(&[0x0d]) {
            Ok(_) => {
                assert!(false, "buffer size less then 8 should return an error");
            }
            Err(_) => {
                assert!(true);
            }
        }
    }

    #[test]
    fn test_parse_op_reply() {
        let buffer = OP_REPLY_BUFFER;
        let (header, next_buffer) = parse_msg_header(&buffer).unwrap();
        let op_query = parse_op_reply(header, next_buffer).unwrap();
        assert_eq!(op_query.response_flags, 8);
        assert_eq!(op_query.cursor_id, 0);
        assert_eq!(op_query.starting_from, 0);
        assert_eq!(op_query.number_returned, 2);
        assert_eq!(op_query.documents[0].contains_key("ismaster"), true);
        assert_eq!(op_query.documents[0].contains_key("maxBsonObjectSize"), true);
        assert_eq!(op_query.documents[0].contains_key("maxMessageSizeBytes"), true);
        assert_eq!(op_query.documents[0].contains_key("maxWriteBatchSize"), true);
        assert_eq!(op_query.documents[0].contains_key("localTime"), true);
        assert_eq!(op_query.documents[0].contains_key("logicalSessionTimeoutMinutes"), true);
        assert_eq!(op_query.documents[0].contains_key("minWireVersion"), true);
        assert_eq!(op_query.documents[0].contains_key("maxWireVersion"), true);
        assert_eq!(op_query.documents[0].contains_key("readOnly"), true);
        assert_eq!(op_query.documents[0].contains_key("ok"), true);
        assert_eq!(op_query.documents[0].get_f64("ok").unwrap(), 1.0);
        assert_eq!(op_query.documents[0].get_bool("readOnly").unwrap(), false);
    }

    #[test]
    fn test_serialize_op_reply() {
        let buffer = OP_REPLY_BUFFER;
        let (header, next_buffer) = parse_msg_header(&buffer).unwrap();
        let op_query = parse_op_reply(header, next_buffer).unwrap();
        let op_reply_buffer = serialize_op_reply(&op_query).unwrap();
        assert_eq!(op_reply_buffer, buffer.to_vec());
    }




    static OP_QUERY_BUFFER: [u8; 269] = [
        0x0d,0x01,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xd4,0x07,0x00,0x00,
        0x00,0x00,0x00,0x00,0x61,0x64,0x6d,0x69,0x6e,0x2e,0x24,0x63,0x6d,0x64,0x00,0x00,
        0x00,0x00,0x00,0x01,0x00,0x00,0x00,0xe6,0x00,0x00,0x00,0x10,0x69,0x73,0x4d,0x61,
        0x73,0x74,0x65,0x72,0x00,0x01,0x00,0x00,0x00,0x03,0x63,0x6c,0x69,0x65,0x6e,0x74,
        0x00,0xcb,0x00,0x00,0x00,0x03,0x61,0x70,0x70,0x6c,0x69,0x63,0x61,0x74,0x69,0x6f,
        0x6e,0x00,0x1d,0x00,0x00,0x00,0x02,0x6e,0x61,0x6d,0x65,0x00,0x0e,0x00,0x00,0x00,
        0x4d,0x6f,0x6e,0x67,0x6f,0x44,0x42,0x20,0x53,0x68,0x65,0x6c,0x6c,0x00,0x00,0x03,
        0x64,0x72,0x69,0x76,0x65,0x72,0x00,0x3a,0x00,0x00,0x00,0x02,0x6e,0x61,0x6d,0x65,
        0x00,0x18,0x00,0x00,0x00,0x4d,0x6f,0x6e,0x67,0x6f,0x44,0x42,0x20,0x49,0x6e,0x74,
        0x65,0x72,0x6e,0x61,0x6c,0x20,0x43,0x6c,0x69,0x65,0x6e,0x74,0x00,0x02,0x76,0x65,
        0x72,0x73,0x69,0x6f,0x6e,0x00,0x06,0x00,0x00,0x00,0x34,0x2e,0x30,0x2e,0x31,0x00,
        0x00,0x03,0x6f,0x73,0x00,0x56,0x00,0x00,0x00,0x02,0x74,0x79,0x70,0x65,0x00,0x07,
        0x00,0x00,0x00,0x44,0x61,0x72,0x77,0x69,0x6e,0x00,0x02,0x6e,0x61,0x6d,0x65,0x00,
        0x09,0x00,0x00,0x00,0x4d,0x61,0x63,0x20,0x4f,0x53,0x20,0x58,0x00,0x02,0x61,0x72,
        0x63,0x68,0x69,0x74,0x65,0x63,0x74,0x75,0x72,0x65,0x00,0x07,0x00,0x00,0x00,0x78,
        0x38,0x36,0x5f,0x36,0x34,0x00,0x02,0x76,0x65,0x72,0x73,0x69,0x6f,0x6e,0x00,0x07,
        0x00,0x00,0x00,0x31,0x38,0x2e,0x32,0x2e,0x30,0x00,0x00,0x00,0x00
    ];



    #[test]
    fn test_parse_u32() {
        let res = parse_u32(&[0x0d,0x01,0x00,0x00]);
        let (num, next_buffer) = res.unwrap();
        assert_eq!(269, num);
        assert_eq!(next_buffer.len(), 0);
    }

    #[test]
    fn test_parse_u32_error() {
        match parse_u32(&[0x0d]) {
            Ok(_) => {
                assert!(false, "buffer size less then 4 should return an error");
            }
            Err(_) => {
                assert!(true);
            }
        }
    }

    #[test]
    fn test_parse_msg_header() {
        let buffer = OP_QUERY_BUFFER;
        let (message_header, _) = parse_msg_header(&buffer).unwrap();
        assert_eq!(message_header.message_length, 269);
        assert_eq!(message_header.request_id, 0);
        assert_eq!(message_header.response_to, 0);
        assert_eq!(get_op_code_value(&message_header.op_code), 2004);
    }

    #[test]
    fn test_parse_cstring() {
        let buffer = OP_QUERY_BUFFER;
        let (_, next_buffer) = parse_msg_header(&buffer).unwrap();
        let (_, next_buffer) = parse_u32(next_buffer).unwrap();
        let (res, _) = parse_cstring(next_buffer).unwrap();
        assert_eq!(res.to_str().unwrap(), "admin.$cmd");
    }

    #[test]
    fn test_parse_cstring_error() {
        let buffer = [0x70,0x70,0x6c,0x69,];
        match parse_cstring(&buffer) {
            Ok((_, _)) => {
                assert!(false, "buffer is not a legal format for cstring");
            },
            Err(_) => {
                assert!(true);
            }
        }
    }

    #[test]
    fn test_parse_bson_document() {
        let buffer = OP_QUERY_BUFFER;
        let (_, next_buffer) = parse_msg_header(&buffer).unwrap();
        let (_, next_buffer) = parse_u32(next_buffer).unwrap();
        let (_, next_buffer) = parse_cstring(next_buffer).unwrap();
        let (_, next_buffer) = parse_u32(next_buffer).unwrap();
        let (_, next_buffer) = parse_u32(next_buffer).unwrap();
        let (doc, _) = parse_bson_document(next_buffer).unwrap();
        assert_eq!(doc.contains_key("isMaster"), true);
        assert_eq!(doc.contains_key("client"), true);
    }

    #[test]
    fn test_parse_op_query() {
        let buffer = OP_QUERY_BUFFER;
        let (header, next_buffer) = parse_msg_header(&buffer).unwrap();
        let op_query = parse_op_query(header, next_buffer).unwrap();
        assert_eq!(op_query.flags, 0);
        assert_eq!(op_query.number_to_skip, 0);
        assert_eq!(op_query.number_to_return, 1);
        assert_eq!(op_query.return_fields_selector, None);
    }
}