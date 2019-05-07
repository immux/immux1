#[derive(Debug)]
pub enum MySQLParserError {
    ParseSqlStatementError(sqlparser::sqlparser::ParserError),
    UnknownCharacterSetValue(u8),
    UnknownIdentifier(u8),
    NoZeroTrailingInCstringBuffer,
    CstringContainZeroByte,
    ParseStringError,
    InputBufferError,
    CannotSetClientStatus,
    CannotSetServerStatusFlags,
}

#[derive(Debug)]
pub enum MySQLSerializeError {
    SerializeAuthPluginDataError,
    SerializeInitialHandshakePacketError,
    PacketSizeTooLarge,
    SerializeAuthSwitchRequestError,
    SerializePluginDataError(std::num::ParseIntError),
    CannotReadClientStatus,
    CannotReadServerStatusFlags,
    LengthEncodedIntegerTooLarge,
    MissingFieldInStruct,
}
