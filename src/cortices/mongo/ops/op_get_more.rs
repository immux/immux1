use std::ffi::CString;

use crate::cortices::mongo::ops::msg_header::MsgHeader;

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-get-more
pub struct OpGetMore {
    // standard message header
    pub header: MsgHeader,

    // 0 - reserved for future use
    pub zero: u32,

    // "dbname.collectionname"
    pub full_collection_name: CString,

    // number of documents to return
    number_to_return: u32,

    // cursorID from the OP_REPLY
    cursor_id: i64,
}
