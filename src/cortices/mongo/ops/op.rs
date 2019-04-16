#![allow(dead_code)]

use crate::cortices::mongo::ops::op_command::OpCommand;
use crate::cortices::mongo::ops::op_command_reply::OpCommandReply;
use crate::cortices::mongo::ops::op_delete::OpDelete;
use crate::cortices::mongo::ops::op_get_more::OpGetMore;
use crate::cortices::mongo::ops::op_insert::OpInsert;
use crate::cortices::mongo::ops::op_kill_cursors::OpKillCursors;
use crate::cortices::mongo::ops::op_msg::OpMsg;
use crate::cortices::mongo::ops::op_query::OpQuery;
use crate::cortices::mongo::ops::op_reply::OpReply;
use crate::cortices::mongo::ops::op_update::OpUpdate;


#[derive(Debug)]
pub enum MongoOp {
    Reply(OpReply),
    Update(OpUpdate),
    Insert(OpInsert),
    Query(OpQuery),
    GetMore(OpGetMore),
    Delete(OpDelete),
    KillCursors(OpKillCursors),
    Command(OpCommand),
    CommandRely(OpCommandReply),
    Msg(OpMsg),
}
