#[derive(Debug)]
pub enum MySQLParserError {
    ParseSqlStatementError(sqlparser::sqlparser::ParserError),
    NotEnoughBufferSize,
    UnknownCharacterSetValue(u8),
    UnknownIdentifier(u8),
    NoZeroTrailingInCstringBuffer,
    CstringContainZeroByte,
    ParseStringError,
    InputBufferError,
}

#[derive(Debug)]
pub enum MySQLSerializeError {
    SerializeAuthPluginDataError,
    PacketSizeTooLarge,
}
