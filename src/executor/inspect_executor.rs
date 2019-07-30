use crate::declarations::commands::{InspectCommand, InspectOutcome, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{Answer, GetEntryInstruction, Instruction};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::get_kv_key;
use crate::storage::core::{CoreStore, ImmuxDBCore};

pub fn execute_inspect(inspect: InspectCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    let instruction = GetEntryInstruction {
        key: get_kv_key(&inspect.grouping, &inspect.id),
    };
    match core.execute(&Instruction::GetEntry(instruction)) {
        Err(error) => return Err(error),
        Ok(answer) => match answer {
            Answer::GetEntryOk(answer) => {
                return Ok(Outcome::Inspect(InspectOutcome {
                    entry: answer.entry,
                }));
            }
            _ => {
                return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                    answer,
                )));
            }
        },
    }
}
