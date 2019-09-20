use crate::declarations::basics::StoreKey;
use crate::declarations::commands::{Outcome, RevertManyCommand, RevertOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::CoreStore;
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataWriteAnswer, DataWriteInstruction, Instruction,
    RevertManyInstruction, RevertTargetSpec,
};

pub fn execute_revert_many(
    revert: RevertManyCommand,
    core: &mut impl CoreStore,
) -> ImmuxResult<Outcome> {
    let instruction = Instruction::DataAccess(DataInstruction::Write(
        DataWriteInstruction::RevertMany(RevertManyInstruction {
            targets: revert
                .specs
                .iter()
                .map(|spec| RevertTargetSpec {
                    key: StoreKey::build(spec.specifier.get_grouping(), spec.specifier.get_id()),
                    height: spec.target_height,
                })
                .collect(),
        }),
    ));
    match core.execute(&instruction) {
        Err(error) => return Err(error),
        Ok(Answer::DataAccess(DataAnswer::Write(DataWriteAnswer::RevertOk(_answer)))) => {
            return Ok(Outcome::RevertMany(RevertOutcome {}));
        }
        Ok(answer) => {
            return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                answer,
            )));
        }
    }
}

#[cfg(test)]
mod revert_many_executor_tests {
    use crate::declarations::basics::{GroupingLabel, StoreKey, UnitId, UnitSpecifier};
    use crate::declarations::commands::{Outcome, RevertCommandTargetSpec, RevertManyCommand};
    use crate::executor::revert_many_executor::execute_revert_many;
    use crate::executor::tests::FixtureCore;
    use crate::storage::instructions::{
        DataInstruction, DataWriteInstruction, Instruction, RevertOkAnswer,
    };
    use crate::storage::vkv::ChainHeight;

    #[test]
    fn test_revert_many() {
        let command = RevertManyCommand {
            specs: vec![RevertCommandTargetSpec {
                specifier: UnitSpecifier::new(GroupingLabel::new(&[1, 2, 3]), UnitId::new(10)),
                target_height: ChainHeight::new(100),
            }],
        };
        let mut core = FixtureCore::new(Box::new(|instruction| match instruction {
            Instruction::DataAccess(DataInstruction::Write(DataWriteInstruction::RevertMany(
                revert_many,
            ))) => {
                assert_eq!(revert_many.targets.len(), 1);
                assert_eq!(
                    revert_many.targets[0].key,
                    StoreKey::new(&[3, 1, 2, 3, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                );
                assert_eq!(revert_many.targets[0].height.as_u64(), 100);
                return Ok(RevertOkAnswer {}.into());
            }
            _ => panic!("Unexpected instruction"),
        }));
        let outcome = execute_revert_many(command, &mut core).unwrap();
        match outcome {
            Outcome::RevertMany(_) => (),
            _ => panic!("Unexpected outcome"),
        }
    }
}
