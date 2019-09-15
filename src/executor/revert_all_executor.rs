use crate::declarations::commands::{Outcome, RevertAllCommand, RevertAllOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::CoreStore;
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataWriteAnswer, DataWriteInstruction, Instruction,
    RevertAllInstruction,
};

pub fn execute_revert_all(
    revert_all: RevertAllCommand,
    core: &mut impl CoreStore,
) -> ImmuxResult<Outcome> {
    let instruction = Instruction::DataAccess(DataInstruction::Write(
        DataWriteInstruction::RevertAll(RevertAllInstruction {
            target_height: revert_all.target_height,
        }),
    ));
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

#[cfg(test)]
mod revert_all_executor_tests {
    use crate::declarations::basics::StoreKey;
    use crate::declarations::commands::{Outcome, RevertAllCommand};
    use crate::executor::revert_all_executor::execute_revert_all;
    use crate::executor::tests::FixtureCore;
    use crate::storage::instructions::{
        DataInstruction, DataWriteInstruction, Instruction, RevertAllOkAnswer,
    };
    use crate::storage::vkv::ChainHeight;

    #[test]
    fn test_revert_all() {
        let command = RevertAllCommand {
            target_height: ChainHeight::new(10),
        };
        let mut core = FixtureCore::new(Box::new(|instruction| match instruction {
            Instruction::DataAccess(DataInstruction::Write(DataWriteInstruction::RevertAll(
                revert_all,
            ))) => {
                assert_eq!(revert_all.target_height.as_u64(), 10);
                return Ok(RevertAllOkAnswer {
                    reverted_keys: [1u8, 2, 3].iter().map(|i| StoreKey::new(&[*i])).collect(),
                }
                .into());
            }
            _ => panic!("Unexpected instruction"),
        }));
        let outcome = execute_revert_all(command, &mut core).unwrap();
        match outcome {
            Outcome::RevertAll(_) => (),
            _ => panic!("Unexpected outcome"),
        }
    }

}
