use crate::config::ConfigError;
use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::mongo::transformer::MongoTransformerError;
use crate::cortices::mysql::error::{MySQLParserError, MySQLSerializeError};
use crate::cortices::tcp::TcpError;
use crate::cortices::unicus::cortex::HttpParsingError;
use crate::cortices::utils::DeserializationError;
use crate::executor::errors::ExecutorError;
use crate::storage::kv::hashmap::HashmapStorageEngineError;
use crate::storage::kv::redis::RedisEngineError;
use crate::storage::kv::rocks::RocksEngineError;
use crate::storage::tkv::TransactionError;
use crate::storage::vkv::VkvError;

#[derive(Debug)]
pub enum ImmuxError {
    RedisEngine(RedisEngineError),
    RocksEngine(RocksEngineError),
    HashmapEngine(HashmapStorageEngineError),

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

impl std::convert::From<MongoParserError> for ImmuxError {
    fn from(error: MongoParserError) -> ImmuxError {
        ImmuxError::MongoParser(error)
    }
}

impl std::convert::From<MongoSerializeError> for ImmuxError {
    fn from(error: MongoSerializeError) -> ImmuxError {
        ImmuxError::MongoSerializer(error)
    }
}

impl std::convert::From<MySQLParserError> for ImmuxError {
    fn from(error: MySQLParserError) -> ImmuxError {
        ImmuxError::MySQLParser(error)
    }
}

impl std::convert::From<MySQLSerializeError> for ImmuxError {
    fn from(error: MySQLSerializeError) -> ImmuxError {
        ImmuxError::MySQLSerializer(error)
    }
}

impl std::convert::From<TcpError> for ImmuxError {
    fn from(error: TcpError) -> ImmuxError {
        ImmuxError::Tcp(error)
    }
}

impl std::convert::From<ConfigError> for ImmuxError {
    fn from(error: ConfigError) -> ImmuxError {
        ImmuxError::Config(error)
    }
}

impl std::convert::From<MongoTransformerError> for ImmuxError {
    fn from(error: MongoTransformerError) -> ImmuxError {
        ImmuxError::MongoTransformer(error)
    }
}

impl std::convert::From<VkvError> for ImmuxError {
    fn from(error: VkvError) -> ImmuxError {
        ImmuxError::VKV(error)
    }
}

impl std::convert::From<DeserializationError> for ImmuxError {
    fn from(error: DeserializationError) -> ImmuxError {
        ImmuxError::Deserialization(error)
    }
}

impl std::convert::From<ExecutorError> for ImmuxError {
    fn from(error: ExecutorError) -> ImmuxError {
        ImmuxError::Executor(error)
    }
}

impl std::convert::From<TransactionError> for ImmuxError {
    fn from(error: TransactionError) -> ImmuxError {
        ImmuxError::Transaction(error)
    }
}

impl std::convert::From<RedisEngineError> for ImmuxError {
    fn from(error: RedisEngineError) -> ImmuxError {
        ImmuxError::RedisEngine(error)
    }
}

impl std::convert::From<RocksEngineError> for ImmuxError {
    fn from(error: RocksEngineError) -> ImmuxError {
        ImmuxError::RocksEngine(error)
    }
}

impl std::convert::From<HashmapStorageEngineError> for ImmuxError {
    fn from(error: HashmapStorageEngineError) -> ImmuxError {
        ImmuxError::HashmapEngine(error)
    }
}

impl std::convert::From<HttpParsingError> for ImmuxError {
    fn from(error: HttpParsingError) -> ImmuxError {
        ImmuxError::HttpParser(error)
    }
}

pub type ImmuxResult<T> = Result<T, ImmuxError>;
