use std::ffi::CString;

use bson::Document;

use crate::cortices::mongo::ops::msg_header::MsgHeader;

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-command
pub struct OpCommand {
    // standard message header
    pub header: MsgHeader,

    // the name of the database to run the command on
    pub database: CString,

    // the name of the command
    pub command_name: CString,

    // a BSON document containing any metadata
    pub metadata: Document,

    // a BSON document containing the command arguments
    pub command_args: Document,

    // a set of zero or more documents
    pub input_docs: Vec<Document>,
}
