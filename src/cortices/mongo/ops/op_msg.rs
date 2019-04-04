#![allow(dead_code)]

use std::ffi::CString;

use bson::Document;

use crate::cortices::mongo::ops::msg_header::MsgHeader;

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#kind-1-document-sequence
pub struct DocumentSequence {
    size: u32,
    identifier: CString,
    documents: Vec<Document>,
}

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#wire-msg-sections
pub enum Session {
    Single(Document),
    Sequence(DocumentSequence),
}

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-msg
pub struct OpMsg {
    // standard message header
    pub header: MsgHeader,

    // message flags
    pub flag_bits: u32,

    // data sections
    pub sections: Vec<Session>,

    // optional CRC-32C checksum
    pub checksum: Option<u32>,
}
