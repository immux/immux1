use crate::cortices::mongo::ops::opcodes::MongoOpCode;

#[derive(Debug)]
pub enum MongoParserError {
    CstringContainZeroByte,
    ParseBsonError(bson::DecoderError),
    NoZeroTrailingInCstringBuffer,
    NotEnoughBufferSize,
    InputBufferError,
    UnimplementedOpCode(MongoOpCode),
    UnknownOpCode(u32),
    UnknownSectionKind,
    OpMsgChecksumMismatch,
}

#[derive(Debug)]
pub enum MongoSerializeError {
    InputObjectError,
    SerializeBsonError(bson::EncoderError),
}
