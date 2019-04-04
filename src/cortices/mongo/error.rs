use crate::cortices::mongo::ops::opcodes::MongoOpCode;

#[derive(Debug)]
pub enum MongoParserError {
    CstringContainZeroByte,
    ParseBsonError(bson::DecoderError),
    NoZeroTrailingInCstringBuffer,
    NotEnoughBufferSize,
    UnimplementedOpCode(MongoOpCode),
    UnknownOpCode(u32),
}
