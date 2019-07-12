use crate::declarations::commands::{NameChainOutcome, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{Answer, Instruction, ReadNamespaceInstruction};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, ImmuxDBCore};

pub fn execute_name_chain(core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    match core.execute(&Instruction::ReadNamespace(ReadNamespaceInstruction {})) {
        Err(error) => return Err(error),
        Ok(answer) => match answer {
            Answer::ReadNamespaceOk(answer) => {
                return Ok(Outcome::NameChain(NameChainOutcome {
                    chain_name: answer.namespace,
                }));
            }
            _ => Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                answer,
            ))),
        },
    }
}
