use crate::config::ConfigError;
use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::mongo::transformer::MongoTransformerError;
use crate::cortices::mysql::error::{MySQLParserError, MySQLSerializeError};
use crate::cortices::tcp::TcpError;
use crate::cortices::unicus::http::HttpParsingError;
use crate::cortices::utils::DeserializationError;
use crate::executor::errors::ExecutorError;
use crate::storage::kv::hashmap::HashmapStorageEngineError;
use crate::storage::kv::redis::RedisEngineError;
use crate::storage::kv::rocks::RocksEngineError;
use crate::storage::tkv::TransactionError;
use crate::storage::vkv::VkvError;

#[derive(Debug)]
pub enum UnumError {
    RedisEngine(RedisEngineError),
    RocksEngine(RocksEngineError),
    HashmapEngine(HashmapStorageEngineError),

    SerializationFail,

    HttpParser(HttpParsingError),

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

    Transaction(TransactionError),
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

impl std::convert::From<TransactionError> for UnumError {
    fn from(error: TransactionError) -> UnumError {
        UnumError::Transaction(error)
    }
}

impl std::convert::From<RedisEngineError> for UnumError {
    fn from(error: RedisEngineError) -> UnumError {
        UnumError::RedisEngine(error)
    }
}

impl std::convert::From<RocksEngineError> for UnumError {
    fn from(error: RocksEngineError) -> UnumError {
        UnumError::RocksEngine(error)
    }
}

impl std::convert::From<HashmapStorageEngineError> for UnumError {
    fn from(error: HashmapStorageEngineError) -> UnumError {
        UnumError::HashmapEngine(error)
    }
}

impl std::convert::From<HttpParsingError> for UnumError {
    fn from(error: HttpParsingError) -> UnumError {
        UnumError::HttpParser(error)
    }
}

pub type UnumResult<T> = Result<T, UnumError>;
