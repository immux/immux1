use bson::Document;

use crate::cortices::mongo::ops::msg_header::MsgHeader;

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#wire-op-commandreply
pub struct OpCommandReply {
    // A standard wire protocol header
    pub header: MsgHeader,

    // A BSON document containing any required metadata
    pub metadata: Document,

    // A BSON document containing the command reply
    pub command_reply: Document,

    // A variable number of BSON documents
    pub output_docs: Document,
}
