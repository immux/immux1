use crate::declarations::basics::UnitId;
use crate::declarations::commands::SelectCondition;
use crate::declarations::errors::ImmuxError;
use crate::storage::instructions::Answer;

#[derive(Debug)]
pub enum ExecutorError {
    UnexpectedAnswerType(Answer),
    CannotSerialize,
    UnimplementedSelectCondition(SelectCondition),
    CannotDeserialize,
    CannotParseJson,
    CannotFindId(UnitId),
    NoneReverseIndex,
}

impl From<ExecutorError> for ImmuxError {
    fn from(error: ExecutorError) -> ImmuxError {
        ImmuxError::Executor(error)
    }
}
