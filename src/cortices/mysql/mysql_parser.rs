use crate::cortices::mysql::error::MySQLParserError;
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use sqlparser::dialect::Dialect;
use sqlparser::sqlast::SQLStatement;
use sqlparser::sqlparser::Parser;

pub fn parse_mysql_op_string_to_ast(
    mysql_op_string: String,
    dialect: &Dialect,
) -> ImmuxResult<Vec<SQLStatement>> {
    match Parser::parse_sql(dialect, mysql_op_string) {
        Err(error) => {
            return Err(ImmuxError::MySQLParser(
                MySQLParserError::ParseSqlStatementError(error),
            ));
        }
        Ok(sql_statements) => Ok(sql_statements),
    }
}

#[cfg(test)]
mod mysql_parser_tests {

    use crate::cortices::mysql::mysql_parser::parse_mysql_op_string_to_ast;
    use sqlparser::dialect::AnsiSqlDialect;
    use sqlparser::sqlast::SQLStatement;

    #[test]
    fn test_parse_mysql_op_string_to_ast() -> Result<(), String> {
        let insert_sql = "INSERT INTO Products \
                          (ProductName, Quantity, Manufacturer) \
                          VALUES \
                          (Pen, 10, Parker);";
        let dialect = AnsiSqlDialect {};
        let sql_statements =
            parse_mysql_op_string_to_ast(insert_sql.to_string(), &dialect).unwrap();
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
                assert_eq!(values[0][0].to_string(), "Pen");
                assert_eq!(values[0][1].to_string(), "10");
                assert_eq!(values[0][2].to_string(), "Parker");
                Ok(())
            }
            _ => Err(String::from("This should be an insert sql statement!")),
        }
    }

    #[test]
    #[should_panic]
    fn test_parse_mysql_op_string_to_ast_error() {
        let nonsense_sql = "Completely nonsense statement";
        let dialect = AnsiSqlDialect {};
        parse_mysql_op_string_to_ast(nonsense_sql.to_string(), &dialect).unwrap();
    }
}
