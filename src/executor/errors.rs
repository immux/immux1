use crate::declarations::basics::GroupingLabel;
use crate::declarations::commands::SelectCondition;
use crate::storage::instructions::Answer;

#[derive(Debug)]
pub enum ExecutorError {
    UnexpectedAnswerType(Answer),
    CannotSerialize,
    UnimplementedSelectCondition(SelectCondition),
    CannotDeserialize,
    UnexpectedNumberType,
    NoIndexedNamesList(GroupingLabel),
}
