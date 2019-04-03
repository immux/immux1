use std::ffi::CString;

use bson::Document;

use crate::cortices::mongo::ops::msg_header::MsgHeader;

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-delete
pub struct OpDelete {
    // standard message header
    pub header: MsgHeader,

    // 0 - reserved for future use
    pub zero: u32,

    // "dbname.collectionname"
    pub full_collection_name: CString,

    // bit vector
    pub flags: u32,

    // query object.
    pub selector: Document,
}
