use crate::declarations::basics::{StoreKey, Unit, UnitContent};
use crate::declarations::commands::{Outcome, RevertManyCommand, RevertOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::executor::insert_executor::get_updates_for_index;
use crate::storage::core::CoreStore;
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction, DataWriteAnswer,
    DataWriteInstruction, GetOneInstruction, Instruction, RevertManyInstruction, RevertTargetSpec,
    SetManyInstruction, SetTargetSpec,
};

pub fn execute_revert_many(
    revert: RevertManyCommand,
    core: &mut impl CoreStore,
) -> ImmuxResult<Outcome> {
    let mut update_for_index: Vec<SetTargetSpec> = Vec::new();
    for revert_spec in &revert.specs {
        let key = StoreKey::build(
            revert_spec.specifier.get_grouping(),
            revert_spec.specifier.get_id(),
        );
        let instruction = Instruction::DataAccess(DataInstruction::Read(
            DataReadInstruction::GetOne(GetOneInstruction {
                key,
                height: Some(revert_spec.target_height),
            }),
        ));
        match core.execute(&instruction) {
            Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                match answer.value.inner() {
                    None => continue,
                    Some(data) => {
                        let content = UnitContent::parse_data(data)?;
                        let unit = Unit {
                            id: revert_spec.specifier.get_id(),
                            content,
                        };
                        let mut set_spect = get_updates_for_index(
                            revert_spec.specifier.get_grouping(),
                            &[unit],
                            core,
                        )?;
                        update_for_index.append(&mut set_spect);
                    }
                }
            }
            _ => continue,
        }
    }
    let batch_update: Instruction = Instruction::DataAccess(DataInstruction::Write(
        DataWriteInstruction::SetMany(SetManyInstruction {
            targets: update_for_index,
        }),
    ));

    core.execute(&batch_update)?;

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
    use crate::declarations::basics::{GroupingLabel, StoreKey, StoreValue, UnitId, UnitSpecifier};
    use crate::declarations::commands::{Outcome, RevertCommandTargetSpec, RevertManyCommand};
    use crate::executor::revert_many_executor::execute_revert_many;
    use crate::executor::tests::FixtureCore;
    use crate::storage::instructions::{
        DataInstruction, DataReadInstruction, DataWriteInstruction, GetOneOkAnswer, Instruction,
        RevertOkAnswer, SetOkAnswer,
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
        let store_key =
            StoreKey::new(&[3, 1, 2, 3, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let mut core = FixtureCore::new(Box::new(|instruction| match instruction {
            Instruction::DataAccess(DataInstruction::Write(DataWriteInstruction::RevertMany(
                revert_many,
            ))) => {
                assert_eq!(revert_many.targets.len(), 1);
                assert_eq!(revert_many.targets[0].key, store_key);
                assert_eq!(revert_many.targets[0].height.as_u64(), 100);
                return Ok(RevertOkAnswer {}.into());
            }
            Instruction::DataAccess(DataInstruction::Read(DataReadInstruction::GetOne(
                get_one,
            ))) => {
                assert_eq!(get_one.key, store_key);
                return Ok(GetOneOkAnswer {
                    value: StoreValue::new(None),
                }
                .into());
            }
            Instruction::DataAccess(DataInstruction::Write(DataWriteInstruction::SetMany(
                set_many,
            ))) => {
                assert!(set_many.targets.is_empty());
                return Ok(SetOkAnswer { count: 0 }.into());
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
