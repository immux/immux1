use crate::declarations::commands::{Outcome, PickChainCommand, PickChainOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DBSystemAnswer, DBSystemInstruction, Instruction, SwitchNamespaceInstruction,
};

pub fn execute_pick_chain(
    pick_chain: PickChainCommand,
    core: &mut ImmuxDBCore,
) -> ImmuxResult<Outcome> {
    let instruction = Instruction::DBSystem(DBSystemInstruction::SwitchNamespace(
        SwitchNamespaceInstruction {
            new_namespace: pick_chain.new_chain_name.into(),
        },
    ));
    match core.execute(&instruction) {
        Err(error) => return Err(error),
        Ok(Answer::DBSystem(DBSystemAnswer::SwitchNamespaceOk(answer))) => {
            return Ok(Outcome::PickChain(PickChainOutcome {
                new_chain_name: answer.new_namespace.into(),
            }))
        }
        Ok(answer) => {
            return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                answer,
            )))
        }
    }
}
