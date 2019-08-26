use crate::declarations::commands::{Outcome, RevertAllCommand, RevertAllOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataWriteAnswer, DataWriteInstruction, Instruction,
    RevertAllInstruction,
};

use crate::storage::core::{CoreStore, ImmuxDBCore};

pub fn execute_revert_all(
    revert_all: RevertAllCommand,
    core: &mut ImmuxDBCore,
) -> ImmuxResult<Outcome> {
    let instruction = Instruction::Data(DataInstruction::Write(DataWriteInstruction::RevertAll(
        RevertAllInstruction {
            target_height: revert_all.target_height,
        },
    )));
    match core.execute(&instruction) {
        Err(error) => return Err(error),
        Ok(Answer::DataAccess(DataAnswer::Write(DataWriteAnswer::RevertAllOk(_answer)))) => {
            return Ok(Outcome::RevertAll(RevertAllOutcome {}));
        }
        Ok(answer) => {
            return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                answer,
            )));
        }
    }
}
