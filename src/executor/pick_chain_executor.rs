use crate::declarations::commands::{Outcome, PickChainCommand, PickChainOutcome};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::declarations::instructions::{Answer, Instruction, SwitchNamespaceInstruction};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, UnumCore};

pub fn execute_pick_chain(
    pick_chain: PickChainCommand,
    core: &mut UnumCore,
) -> UnumResult<Outcome> {
    let instruction = SwitchNamespaceInstruction {
        new_namespace: pick_chain.new_chain_name,
    };
    match core.execute(&Instruction::SwitchNamespace(instruction)) {
        Err(error) => return Err(error),
        Ok(answer) => match answer {
            Answer::SwitchNamespaceOk(answer) => {
                return Ok(Outcome::PickChain(PickChainOutcome {
                    new_chain_name: answer.new_namespace,
                }))
            }
            _ => Err(UnumError::Executor(ExecutorError::UnexpectedAnswerType(
                answer,
            ))),
        },
    }
}
