use crate::cortices::mongo::ops::op::MongoOp;
use crate::cortices::mongo::ops::op_reply::OpReply;
use crate::declarations::errors::UnumResult;
use crate::declarations::instructions::{Answer, GetInstruction, Instruction};

pub fn transform_mongo_op(op: &MongoOp) -> UnumResult<Instruction> {
    match op {
        MongoOp::Insert(_insert) => Ok(Instruction::Get(GetInstruction { targets: vec![] })),
        MongoOp::Query(_query) => unimplemented!(),
        _ => unimplemented!(),
    }
}

pub fn transform_answer_for_mongo(answer: &Answer) -> UnumResult<OpReply> {
    match answer {
        _ => unimplemented!(),
    }
}
