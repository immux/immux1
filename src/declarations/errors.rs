use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::tcp::TcpError;

#[derive(Debug)]
pub enum UnumError {
    InitializationFail,
    ReadError,
    WriteError,

    SerializationFail,
    DeserializationFail,

    UrlParseError,

    Tcp(TcpError),

    MongoParser(MongoParserError),
    MongoSerializer(MongoSerializeError)
}

impl std::convert::From<MongoParserError> for UnumError {
    fn from(error: MongoParserError) -> UnumError {
        UnumError::MongoParser(error)
    }
}

impl std::convert::From<TcpError> for UnumError {
    fn from(error: TcpError) -> UnumError {
        UnumError::Tcp(error)
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
