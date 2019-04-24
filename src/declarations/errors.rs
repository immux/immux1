use crate::config::ConfigError;
use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::mysql::error::{MySQLParserError, MySQLSerializeError};
use crate::cortices::tcp::TcpError;

#[derive(Debug)]
pub enum UnumError {
    InitializationFail,
    ReadError,
    WriteError,

    SerializationFail,
    DeserializationFail,

    UrlParseError,

    Config(ConfigError),

    Tcp(TcpError),

    MongoParser(MongoParserError),
    MongoSerializer(MongoSerializeError),

    MySQLParser(MySQLParserError),
    MySQLSerializer(MySQLSerializeError),
}

impl std::convert::From<MongoParserError> for UnumError {
    fn from(error: MongoParserError) -> UnumError {
        UnumError::MongoParser(error)
    }
}

impl std::convert::From<MongoSerializeError> for UnumError {
    fn from(error: MongoSerializeError) -> UnumError {
        UnumError::MongoSerializer(error)
    }
}

impl std::convert::From<MySQLParserError> for UnumError {
    fn from(error: MySQLParserError) -> UnumError {
        UnumError::MySQLParser(error)
    }
}

impl std::convert::From<MySQLSerializeError> for UnumError {
    fn from(error: MySQLSerializeError) -> UnumError {
        UnumError::MySQLSerializer(error)
    }
}

impl std::convert::From<TcpError> for UnumError {
    fn from(error: TcpError) -> UnumError {
        UnumError::Tcp(error)
    }
}

impl std::convert::From<ConfigError> for UnumError {
    fn from(error: ConfigError) -> UnumError {
        UnumError::Config(error)
    }
}

pub fn explain_error(error: UnumError) -> &'static str {
    match error {
        UnumError::InitializationFail => "initialization failed",
        UnumError::ReadError => "read error",
        UnumError::WriteError => "write error",
        UnumError::DeserializationFail => "deserialization failed",
        UnumError::SerializationFail => "serialization failed",
        UnumError::UrlParseError => "url parse error",
        _ => "Error with unspecified explanation",
    }
}

pub type UnumResult<T> = Result<T, UnumError>;
