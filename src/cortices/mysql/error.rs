#[derive(Debug)]
pub enum MySQLParserError {
    ParseSqlStatementError(sqlparser::sqlparser::ParserError),
}

#[derive(Debug)]
pub enum MySQLSerializeError {
    SerializeAuthPluginDataError,
    PacketSizeTooLarge,
}
