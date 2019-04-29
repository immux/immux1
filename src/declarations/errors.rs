use crate::config::ConfigError;
use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::mongo::transformer::MongoTransformerError;
use crate::cortices::mysql::error::{MySQLParserError, MySQLSerializeError};
use crate::cortices::tcp::TcpError;
use crate::cortices::utils::DeserializationError;
use crate::executor::execute::ExecutorError;
use crate::storage::vkv::VkvError;

#[derive(Debug)]
pub enum UnumError {
    InitializationFail,
    ReadError,
    WriteError,

    SerializationFail,
    DeserializationFail,

    UrlParseError,

    VKV(VkvError),

    Config(ConfigError),

    Tcp(TcpError),

    Executor(ExecutorError),

    Deserialization(DeserializationError),

    MongoParser(MongoParserError),
    MongoSerializer(MongoSerializeError),
    MongoTransformer(MongoTransformerError),

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

impl std::convert::From<MongoTransformerError> for UnumError {
    fn from(error: MongoTransformerError) -> UnumError {
        UnumError::MongoTransformer(error)
    }
}

impl std::convert::From<VkvError> for UnumError {
    fn from(error: VkvError) -> UnumError {
        UnumError::VKV(error)
    }
}

impl std::convert::From<DeserializationError> for UnumError {
    fn from(error: DeserializationError) -> UnumError {
        UnumError::Deserialization(error)
    }
}

impl std::convert::From<ExecutorError> for UnumError {
    fn from(error: ExecutorError) -> UnumError {
        UnumError::Executor(error)
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
