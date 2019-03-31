use std::ffi::CString;
extern crate bson;

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
    pub query: bson::Document,
    pub return_fields_selector: Option<bson::Document>,
}
