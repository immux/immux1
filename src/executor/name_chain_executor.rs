use crate::declarations::commands::{NameChainOutcome, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::CoreStore;
use crate::storage::instructions::{
    Answer, DBSystemAnswer, DBSystemInstruction, Instruction, ReadNamespaceInstruction,
};

pub fn execute_name_chain(core: &mut impl CoreStore) -> ImmuxResult<Outcome> {
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

#[cfg(test)]
mod name_chain_executor_tests {
    use crate::declarations::commands::Outcome;
    use crate::executor::name_chain_executor::execute_name_chain;
    use crate::executor::tests::FixtureCore;
    use crate::storage::instructions::{ReadNamespaceOkAnswer, StoreNamespace};

    #[test]
    fn test_name_chain() {
        let mut core = FixtureCore::new(Box::new(|_instruction| {
            Ok(ReadNamespaceOkAnswer {
                namespace: StoreNamespace::new(&[0, 1, 2, 3]),
            }
            .into())
        }));
        let outcome = execute_name_chain(&mut core).unwrap();
        match outcome {
            Outcome::NameChain(outcome) => assert_eq!(outcome.chain_name.as_bytes(), &[0, 1, 2, 3]),
            _ => panic!("Unexpected outcome"),
        }
    }
}
