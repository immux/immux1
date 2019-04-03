use crate::declarations::errors::{UnumResult, UnumError};
use crate::cortices::mongo::error::MongoParserError;
use crate::cortices::mongo::ops::msg_header::parse_msg_header;
use crate::cortices::mongo::ops::op::MongoOp;
use crate::cortices::mongo::ops::op_query::parse_op_query;
use crate::cortices::mongo::ops::opcodes::MongoOpCode;

pub fn parse_mongo_incoming_bytes(buffer: &[u8]) -> UnumResult<MongoOp> {
    println!("Total {} bytes were read", buffer.len());
    match parse_msg_header(buffer) {
        Err(error) => Err(error),
        Ok((header, remaining_buffer)) => {
            println!("{:#?}", &header);
            match header.op_code {
                MongoOpCode::OpQuery => match parse_op_query(header, remaining_buffer) {
                    Err(error) => Err(error),
                    Ok(op) => Ok(MongoOp::Query(op)),
                },
                _ => Err(UnumError::MongoParser(MongoParserError::UnimplementedOpCode(header.op_code))),
            }
        }
    }
}
