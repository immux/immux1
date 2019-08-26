use crate::declarations::basics::StoreKey;
use crate::declarations::commands::{Outcome, RevertCommand, RevertOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataWriteAnswer, DataWriteInstruction, Instruction,
    RevertManyInstruction, RevertTargetSpec,
};

pub fn execute_revert_one(revert: RevertCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    let instruction = Instruction::Data(DataInstruction::Write(DataWriteInstruction::RevertMany(
        RevertManyInstruction {
            targets: revert
                .specs
                .iter()
                .map(|spec| RevertTargetSpec {
                    key: StoreKey::build(spec.specifier.get_grouping(), spec.specifier.get_id()),
                    height: spec.target_height,
                })
                .collect(),
        },
    )));
    match core.execute(&instruction) {
        Err(error) => return Err(error),
        Ok(Answer::DataAccess(DataAnswer::Write(DataWriteAnswer::RevertOk(_answer)))) => {
            return Ok(Outcome::Revert(RevertOutcome {}));
        }
        Ok(answer) => {
            return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                answer,
            )));
        }
    }
}
