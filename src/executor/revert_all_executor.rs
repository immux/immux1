use crate::declarations::commands::{Outcome, RevertAllCommand, RevertAllOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{Answer, AtomicRevertAllInstruction, Instruction};
use crate::executor::errors::ExecutorError;

use crate::storage::core::{CoreStore, ImmuxDBCore};

pub fn execute_revert_all(
    revert_all: RevertAllCommand,
    core: &mut ImmuxDBCore,
) -> ImmuxResult<Outcome> {
    let instruction = AtomicRevertAllInstruction {
        target_height: revert_all.target_height,
    };
    match core.execute(&Instruction::AtomicRevertAll(instruction)) {
        Err(error) => return Err(error),
        Ok(answer) => match answer {
            Answer::RevertAllOk(_answer) => {
                return Ok(Outcome::RevertAll(RevertAllOutcome {}));
            }
            _ => {
                return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                    answer,
                )));
            }
        },
    }
}
