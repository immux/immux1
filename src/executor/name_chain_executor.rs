use crate::declarations::commands::{NameChainOutcome, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DBSystemAnswer, DBSystemInstruction, Instruction, ReadNamespaceInstruction,
};

pub fn execute_name_chain(core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    let instruction = Instruction::DBSystem(DBSystemInstruction::ReadNamespace(
        ReadNamespaceInstruction {},
    ));
    match core.execute(&instruction) {
        Err(error) => return Err(error),
        Ok(Answer::DBSystem(DBSystemAnswer::ReadNamespaceOk(answer))) => {
            return Ok(Outcome::NameChain(NameChainOutcome {
                chain_name: answer.namespace.into(),
            }));
        }
        Ok(answer) => {
            return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                answer,
            )))
        }
    }
}
