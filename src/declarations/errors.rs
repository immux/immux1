use std::fmt::Formatter;

use crate::config::ConfigError;
use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::mongo::transformer::MongoTransformerError;
use crate::cortices::mysql::error::{MySQLParserError, MySQLSerializeError};
use crate::cortices::tcp::TcpError;
use crate::cortices::unicus::cortex::HttpParsingError;
use crate::cortices::utils::DeserializationError;
use crate::declarations::basics::id_list::IdListError;
use crate::declarations::basics::property_names::PropertyNameListError;
use crate::declarations::basics::store_value::StoreValueError;
use crate::declarations::basics::unit_id::UnitIdError;
use crate::declarations::basics::{StoreKeyError, UnitContentError};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::ReverseIndexError;
use crate::storage::kv::KVError;
use crate::storage::tkv::TransactionError;
use crate::storage::vkv::{ChainHeightError, VkvError};

#[derive(Debug)]
pub enum ImmuxError {
    HttpParser(HttpParsingError),
    HttpResponse(std::io::Error),

    VKV(VkvError),
    KV(KVError),

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

    UnitContentProcessing(UnitContentError),
    ReverseIndexProcessing(ReverseIndexError),
    UnitId(UnitIdError),
    StoreKey(StoreKeyError),
    StoreValue(StoreValueError),
    ChainHeight(ChainHeightError),
    PropertyName(PropertyNameListError),
    IdList(IdListError),
}

impl std::fmt::Display for ImmuxError {
    fn fmt(&self, _formatter: &mut Formatter<'_>) -> std::fmt::Result {
        return Ok(());
    }
}

impl std::error::Error for ImmuxError {}

impl From<MongoParserError> for ImmuxError {
    fn from(error: MongoParserError) -> ImmuxError {
        ImmuxError::MongoParser(error)
    }
}

impl From<MongoSerializeError> for ImmuxError {
    fn from(error: MongoSerializeError) -> ImmuxError {
        ImmuxError::MongoSerializer(error)
    }
}

impl From<MySQLParserError> for ImmuxError {
    fn from(error: MySQLParserError) -> ImmuxError {
        ImmuxError::MySQLParser(error)
    }
}

impl From<MySQLSerializeError> for ImmuxError {
    fn from(error: MySQLSerializeError) -> ImmuxError {
        ImmuxError::MySQLSerializer(error)
    }
}

impl From<TcpError> for ImmuxError {
    fn from(error: TcpError) -> ImmuxError {
        ImmuxError::Tcp(error)
    }
}

impl From<ConfigError> for ImmuxError {
    fn from(error: ConfigError) -> ImmuxError {
        ImmuxError::Config(error)
    }
}

impl From<MongoTransformerError> for ImmuxError {
    fn from(error: MongoTransformerError) -> ImmuxError {
        ImmuxError::MongoTransformer(error)
    }
}

impl From<VkvError> for ImmuxError {
    fn from(error: VkvError) -> ImmuxError {
        ImmuxError::VKV(error)
    }
}

impl From<DeserializationError> for ImmuxError {
    fn from(error: DeserializationError) -> ImmuxError {
        ImmuxError::Deserialization(error)
    }
}

impl From<TransactionError> for ImmuxError {
    fn from(error: TransactionError) -> ImmuxError {
        ImmuxError::Transaction(error)
    }
}

impl From<HttpParsingError> for ImmuxError {
    fn from(error: HttpParsingError) -> ImmuxError {
        ImmuxError::HttpParser(error)
    }
}

impl From<UnitContentError> for ImmuxError {
    fn from(error: UnitContentError) -> ImmuxError {
        ImmuxError::UnitContentProcessing(error)
    }
}

impl From<UnitIdError> for ImmuxError {
    fn from(error: UnitIdError) -> Self {
        ImmuxError::UnitId(error)
    }
}

impl From<ReverseIndexError> for ImmuxError {
    fn from(error: ReverseIndexError) -> ImmuxError {
        ImmuxError::ReverseIndexProcessing(error)
    }
}

pub type ImmuxResult<T> = Result<T, ImmuxError>;
