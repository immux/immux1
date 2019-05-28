use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::msg_header::get_msg_header_op_code;
use crate::cortices::mongo::ops::op::MongoOp;
use crate::cortices::mongo::ops::op_msg::parse_op_msg;
use crate::cortices::mongo::ops::op_query::parse_op_query;
use crate::cortices::mongo::ops::opcodes::MongoOpCode;
use crate::declarations::errors::{ImmuxError, ImmuxResult};

pub fn parse_mongo_incoming_bytes(buffer: &[u8]) -> ImmuxResult<MongoOp> {
    let mut index: usize = 0;
    match get_msg_header_op_code(buffer) {
        Err(error) => Err(error),
        Ok(op_code) => match op_code {
            MongoOpCode::OpQuery => match parse_op_query(&buffer[index..]) {
                Err(error) => Err(error),
                Ok(op) => Ok(MongoOp::Query(op)),
            },
            MongoOpCode::OpMsg => match parse_op_msg(&buffer[index..]) {
                Err(error) => Err(error),
                Ok(op) => Ok(MongoOp::Msg(op)),
            },
            _ => Err(ImmuxError::MongoParser(
                MongoParserError::UnimplementedOpCode(op_code),
            )),
        },
    }
}
