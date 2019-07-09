use crate::declarations::commands::SelectCondition;
use crate::declarations::instructions::Answer;

#[derive(Debug)]
pub enum ExecutorError {
    UnexpectedAnswerType(Answer),
    CannotSerialize,
    UnimplementedSelectCondition(SelectCondition),
    CannotDeserialize,
    UnexpectedNumberType,
}
