use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::msg_header::parse_msg_header;
use crate::cortices::mongo::ops::op::MongoOp;
use crate::cortices::mongo::ops::op_msg::parse_op_msg;
use crate::cortices::mongo::ops::op_query::parse_op_query;
use crate::cortices::mongo::ops::opcodes::MongoOpCode;
use crate::declarations::errors::{UnumError, UnumResult};

pub fn parse_mongo_incoming_bytes(buffer: &[u8]) -> UnumResult<MongoOp> {
    match parse_msg_header(buffer) {
        Err(error) => Err(error),
        Ok((header, remaining_buffer)) => match header.op_code {
            MongoOpCode::OpQuery => match parse_op_query(header, remaining_buffer) {
                Err(error) => Err(error),
                Ok(op) => Ok(MongoOp::Query(op)),
            },
            MongoOpCode::OpMsg => match parse_op_msg(header, remaining_buffer) {
                Err(error) => Err(error),
                Ok(op) => Ok(MongoOp::Msg(op)),
            },
            _ => Err(UnumError::MongoParser(
                MongoParserError::UnimplementedOpCode(header.op_code),
            )),
        },
    }
}
