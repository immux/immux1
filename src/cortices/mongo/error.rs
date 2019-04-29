use crate::cortices::mongo::ops::opcodes::MongoOpCode;

#[derive(Debug)]
pub enum MongoParserError {
    ParseBsonError(bson::DecoderError),
    InputBufferError,
    UnimplementedOpCode(MongoOpCode),
    UnknownOpCode(u32),
    UnknownSectionKind,
}

#[derive(Debug)]
pub enum MongoSerializeError {
    InputObjectError,
    SerializeBsonError(bson::EncoderError),
}
