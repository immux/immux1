use crate::declarations::commands::{Outcome, RevertCommand, RevertOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{
    Answer, AtomicRevertInstruction, Instruction, RevertTargetSpec,
};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::get_kv_key;
use crate::storage::core::{CoreStore, ImmuxDBCore};

pub fn execute_revert_one(revert: RevertCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    let instruction = AtomicRevertInstruction {
        targets: revert
            .specs
            .iter()
            .map(|spec| RevertTargetSpec {
                key: get_kv_key(&revert.grouping, &spec.id),
                height: spec.target_height,
            })
            .collect(),
    };
    match core.execute(&Instruction::AtomicRevert(instruction)) {
        Err(error) => return Err(error),
        Ok(answer) => match answer {
            Answer::RevertOk(answer) => {
                return Ok(Outcome::Revert(RevertOutcome {
                    count: answer.items.len() as u64,
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
