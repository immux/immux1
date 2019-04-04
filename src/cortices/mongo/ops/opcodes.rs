/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#request-opcodes
pub const MONGO_OP_REPLY_CODE: u32 = 1;
pub const MONGO_OP_UPDATE_CODE: u32 = 2001;
pub const MONGO_OP_INSERT_CODE: u32 = 2002;
pub const MONGO_OP_QUERY_CODE: u32 = 2004;
pub const MONGO_OP_GET_MORE_CODE: u32 = 2005;
pub const MONGO_OP_DELETE_CODE: u32 = 2006;
pub const MONGO_OP_KILL_CURSORS_CODE: u32 = 2007;
pub const MONGO_OP_COMMAND_CODE: u32 = 2010;
pub const MONGO_OP_COMMAND_REPLY_CODE: u32 = 2011;
pub const MONGO_OP_MSG_CODE: u32 = 2013;

#[derive(Debug)]
#[repr(u32)]
pub enum MongoOpCode {
    OpReply = MONGO_OP_REPLY_CODE,
    OpUpdate = MONGO_OP_UPDATE_CODE,
    OpInsert = MONGO_OP_INSERT_CODE,
    //    RESERVED = 2003
    OpQuery = MONGO_OP_QUERY_CODE,
    OpGetMore = MONGO_OP_GET_MORE_CODE,
    OpDelete = MONGO_OP_DELETE_CODE,
    OpKillCursors = MONGO_OP_KILL_CURSORS_CODE,
    OpCommand = MONGO_OP_COMMAND_CODE,
    OpCommandReply = MONGO_OP_COMMAND_REPLY_CODE,
    OpMsg = MONGO_OP_MSG_CODE,
}
