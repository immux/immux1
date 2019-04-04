use bson::Document;

use crate::cortices::mongo::ops::msg_header::MsgHeader;

/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-reply
#[derive(Debug)]
pub struct OpReply {
    // standard message header
    pub header: MsgHeader,

    // bit vector
    pub response_flags: u32,

    // cursor id if client needs to do get more's
    pub cursor_id: u32,

    // where in the cursor this reply is starting
    pub starting_from: u32,

    // number of documents in the reply
    pub number_returned: u32,

    // documents
    pub documents: Vec<Document>,
}

pub fn serialize_op_reply(_op: &OpReply) -> Vec<u8> {
    return vec![]; // TODO(#10)
}
