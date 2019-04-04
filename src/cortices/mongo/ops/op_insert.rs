use std::ffi::CString;

use bson::Document;

use crate::cortices::mongo::ops::msg_header::MsgHeader;

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-insert
pub struct OpInsert {
    // standard message header
    pub header: MsgHeader,

    // bit vector
    pub flags: u32,

    // "dbname.collectionname"
    pub full_collection_name: CString,

    // one or more documents to insert into the collection
    pub documents: Vec<Document>,
}
