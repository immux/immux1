use crate::declarations::commands::{Outcome, PickChainCommand, PickChainOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::CoreStore;
use crate::storage::instructions::{
    Answer, DBSystemAnswer, DBSystemInstruction, Instruction, SwitchNamespaceInstruction,
};

pub fn execute_pick_chain(
    pick_chain: PickChainCommand,
    core: &mut impl CoreStore,
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

#[cfg(test)]
mod pick_chain_executor_tests {
    use crate::declarations::basics::ChainName;
    use crate::declarations::commands::{Outcome, PickChainCommand};
    use crate::executor::pick_chain_executor::execute_pick_chain;
    use crate::executor::tests::FixtureCore;
    use crate::storage::instructions::{DBSystemInstruction, Instruction, SwitchNamespaceOkAnswer};

    #[test]
    fn test_pick_chain() {
        let name_bytes = [0xff, 0x00];
        let mut core = FixtureCore::new(Box::new(|instruction| match instruction {
            Instruction::DBSystem(DBSystemInstruction::SwitchNamespace(switch_namespace)) => {
                Ok(SwitchNamespaceOkAnswer {
                    new_namespace: switch_namespace.new_namespace.to_owned(),
                }
                .into())
            }
            _ => panic!("Unimplemented fixture instruction {:?}", instruction),
        }));
        let command = PickChainCommand {
            new_chain_name: ChainName::new(&name_bytes),
        };
        let outcome = execute_pick_chain(command, &mut core).unwrap();
        match outcome {
            Outcome::PickChain(outcome) => {
                assert_eq!(outcome.new_chain_name.as_bytes(), &name_bytes)
            }
            _ => panic!("Unexpected outcome"),
        }
    }

}
