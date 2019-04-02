use std::ffi::CString;
use bson::Document;

pub struct MsgHeader {
    pub message_length: u32,
    pub request_id: u32,
    pub response_to: u32,
    pub op_code: u32,
}

pub struct OpQuery {
    pub message_header: MsgHeader,
    pub flags: u32,
    pub full_collection_name: CString,
    pub number_to_skip: u32,
    pub number_to_return: u32,
    pub query: Document,
    pub return_fields_selector: Option<bson::Document>,
}

pub struct OpReply {
    pub message_header: MsgHeader,
    pub response_flags: u32,
    pub cursor_id: u64,
    pub starting_from: u32,
    pub number_returned: u32,
    pub documents: Vec<Document>,
}
