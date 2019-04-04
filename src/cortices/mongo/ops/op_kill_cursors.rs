use crate::cortices::mongo::ops::msg_header::MsgHeader;

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-kill-cursors
pub struct OpKillCursors {
    // standard message header
    pub header: MsgHeader,

    // 0 - reserved for future use
    pub zero: u32,

    // number of cursorIDs in message
    pub number_of_cursor_ids: u32,

    // sequence of cursorIDs to close
    pub cursor_ids: Vec<i64>,
}
