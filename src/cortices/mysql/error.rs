#[derive(Debug)]
pub enum MySQLParserError {
    ParseSqlStatementError(sqlparser::sqlparser::ParserError),
}
