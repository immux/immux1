use std::ffi::CString;

use bson::Document;

use crate::cortices::mongo::ops::msg_header::MsgHeader;

/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-update
#[derive(Debug)]
pub struct OpUpdate {
    // standard message header
    pub header: MsgHeader,

    // 0 - reserved for future use
    pub zero: u32,

    // "dbname.collectionname"
    pub full_collection_name: CString,

    // bit vector.
    pub flags: u32,

    // the query to select the document
    pub selector: Document,

    // specification of the update to perform
    pub update: Document,
}
