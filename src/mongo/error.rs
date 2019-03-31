extern crate bson;

#[derive(Debug)]
pub enum ParseError {
    CstringContainZeroByte,
    ParseBsonError(bson::DecoderError),
    NoZeroTrailingInCstringBuffer,
    NotEnoughBufferSize,
}
