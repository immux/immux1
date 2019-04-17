use crate::cortices::mysql::error::MySQLParserError;
use crate::declarations::errors::{UnumError, UnumResult};
use sqlparser::dialect::Dialect;
use sqlparser::sqlast::SQLStatement;
use sqlparser::sqlparser::Parser;

fn parse_mysql_statement(
    mysql_statement: String,
    dialect: &Dialect,
) -> UnumResult<Vec<SQLStatement>> {
    match Parser::parse_sql(dialect, mysql_statement) {
        Err(error) => {
            return Err(UnumError::MySQLParser(
                MySQLParserError::ParseSqlStatementError(error),
            ));
        }
        Ok(sql_statements) => Ok(sql_statements),
    }
}

#[cfg(test)]
mod mysql_parser_tests {

    use crate::cortices::mysql::mysql_parser::parse_mysql_statement;
    use sqlparser::dialect::AnsiSqlDialect;
    use sqlparser::sqlast::SQLStatement;

    #[test]
    fn test_parse_mysql_statement() {
        let insert_sql = "INSERT INTO Products \
                          (ProductName, Quantity, Manufacturer) \
                          VALUES \
                          ('Pen', 10, 'Parker');";
        let dialect = AnsiSqlDialect {};
        let sql_statements = parse_mysql_statement(insert_sql.to_string(), &dialect).unwrap();
        assert_eq!(sql_statements.len(), 1);
        match &sql_statements[0] {
            SQLStatement::SQLInsert {
                table_name,
                columns,
                values,
            } => {
                assert_eq!(table_name.to_string(), "Products");
                assert_eq!(columns[0].to_string(), "ProductName");
                assert_eq!(columns[1].to_string(), "Quantity");
                assert_eq!(columns[2].to_string(), "Manufacturer");
                assert_eq!(values.len(), 1);
                assert_eq!(values[0][1].to_string(), "10");
            }
            _ => assert!(false, "This should be an insert sql statement!"),
        }
    }

    #[test]
    fn test_parse_mysql_statement_error() {
        let nonsense_sql = "Completely nonsense statement";
        let dialect = AnsiSqlDialect {};
        match parse_mysql_statement(nonsense_sql.to_string(), &dialect) {
            Err(_) => assert!(true),
            Ok(_) => assert!(
                false,
                "This a nonsense sql statement, it should not get parsed"
            ),
        }
    }
}
