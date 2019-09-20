use crate::declarations::basics::{StoreKey, UnitContent};
use crate::declarations::commands::{InspectCommand, InspectOutcome, Inspection, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::CoreStore;
use crate::storage::instructions::{
    Answer, DataAnswer, DataReadAnswer, GetJournalInstruction, GetOneInstruction, Instruction,
};

pub fn execute_inspect(inspect: InspectCommand, core: &mut impl CoreStore) -> ImmuxResult<Outcome> {
    let store_key = StoreKey::from(inspect.specifier);
    let get_journal: Instruction = GetJournalInstruction {
        key: store_key.clone(),
    }
    .into();
    match core.execute(&get_journal) {
        Err(error) => return Err(error),
        Ok(answer) => match answer {
            Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetJournalOk(journal_answer))) => {
                let mut inspections: Vec<Inspection> = Vec::new();
                for height in journal_answer.journal.update_heights.iter() {
                    let get_value = GetOneInstruction {
                        height: Some(height),
                        key: store_key.clone(),
                    };
                    match core.execute(&get_value.into()) {
                        Err(error) => return Err(error),
                        Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(
                            answer,
                        )))) => {
                            let inspection = match answer.value.inner() {
                                None => Inspection {
                                    height,
                                    content: None,
                                },
                                Some(data) => Inspection {
                                    height,
                                    content: Some(UnitContent::parse_data(data)?),
                                },
                            };
                            inspections.push(inspection);
                        }
                        Ok(answer) => {
                            return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                                answer,
                            )))
                        }
                    }
                }
                let outcome = Outcome::Inspect(InspectOutcome { inspections });
                return Ok(outcome);
            }
            _ => {
                return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                    answer,
                )));
            }
        },
    }
}

#[cfg(test)]
mod inspect_executor_tests {
    use std::collections::HashMap;

    use crate::declarations::basics::{
        GroupingLabel, StoreKey, StoreValue, UnitContent, UnitId, UnitSpecifier,
    };
    use crate::declarations::commands::{InspectCommand, Inspection, Outcome};
    use crate::declarations::errors::ImmuxError;
    use crate::executor::inspect_executor::execute_inspect;
    use crate::executor::tests::FixtureCore;
    use crate::storage::instructions::{
        DataInstruction, DataReadInstruction, GetJournalOkAnswer, GetOneOkAnswer, Instruction,
    };
    use crate::storage::vkv::{ChainHeight, HeightList, UnitJournal, VkvError};

    #[test]
    fn test_cannot_get_journal() {
        let mut core = FixtureCore::new(Box::new(|_instruction| {
            return Err(VkvError::MissingJournal(StoreKey::from("hello")).into());
        }));
        let command = InspectCommand {
            specifier: UnitSpecifier::new(GroupingLabel::from("grouping"), UnitId::new(1)),
        };
        match execute_inspect(command, &mut core) {
            Err(ImmuxError::VKV(VkvError::MissingJournal(key))) => {
                assert_eq!(key, StoreKey::from("hello"))
            }
            Err(error) => panic!("Unexpected error {}", error),
            Ok(_) => panic!("Should get missing journal when core returns that error"),
        }
    }

    /// Simulated core with a {key: updates} table, and inspect a row and compare outcome with the
    /// artificial journal
    #[test]
    fn test_get_journal() {
        let grouping = GroupingLabel::from("grouping");

        let updates_table: HashMap<StoreKey, Vec<(ChainHeight, StoreValue)>> = vec![
            (1, vec![(1, "A"), (2, "B"), (4, "X")]),
            (2, vec![(3, "hello"), (6, "world")]),
            (3, vec![(5, "x"), (7, "x"), (8, "x")]),
        ]
        .into_iter()
        .map(|(id, data)| {
            let unit_id = UnitId::new(id);
            let specifier = UnitSpecifier::new(grouping.clone(), unit_id);
            let key = StoreKey::from(specifier);
            let typed_data = data
                .into_iter()
                .map(|(h, s)| {
                    (
                        ChainHeight::new(h),
                        StoreValue::new(Some(UnitContent::String(s.to_string()).marshal())),
                    )
                })
                .collect();
            (key, typed_data)
        })
        .collect();

        let mut core = FixtureCore::new(Box::new(|instruction| match instruction {
            Instruction::DataAccess(DataInstruction::Read(DataReadInstruction::GetJournal(
                get_journal,
            ))) => match updates_table.get(&get_journal.key) {
                None => Err(VkvError::MissingJournal(get_journal.key.to_owned()).into()),
                Some(data) => {
                    let value = data.last().unwrap().1.to_owned();
                    let heights: Vec<ChainHeight> = data
                        .iter()
                        .map(|(height, _value)| *height)
                        .to_owned()
                        .collect();
                    let update_heights = HeightList::new(&heights);
                    Ok(GetJournalOkAnswer {
                        journal: UnitJournal {
                            value,
                            update_heights,
                        },
                    }
                    .into())
                }
            },
            Instruction::DataAccess(DataInstruction::Read(DataReadInstruction::GetOne(
                get_one,
            ))) => Ok(GetOneOkAnswer {
                value: match updates_table.get(&get_one.key) {
                    None => return Err(VkvError::MissingJournal(get_one.key.to_owned()).into()),
                    Some(data) => match get_one.height {
                        None => data.last().unwrap().1.to_owned(),
                        Some(target_height) => data
                            .iter()
                            .find_map(|(height, value)| {
                                if height.as_u64() == target_height.as_u64() {
                                    Some(value.to_owned())
                                } else {
                                    None
                                }
                            })
                            .unwrap(),
                    },
                },
            }
            .into()),
            instruction => panic!("Unexpected instruction: {:?}", instruction),
        }));
        let command = InspectCommand {
            specifier: UnitSpecifier::new(grouping.clone(), UnitId::new(2)),
        };
        match execute_inspect(command, &mut core) {
            Err(error) => panic!("Inspect failed: {:?}", error),
            Ok(Outcome::Inspect(outcome)) => {
                assert_eq!(outcome.inspections.len(), 2);
                let expected_first = Inspection {
                    height: ChainHeight::new(3),
                    content: Some(UnitContent::String("hello".to_string())),
                };
                let expected_second = Inspection {
                    height: ChainHeight::new(6),
                    content: Some(UnitContent::String("world".to_string())),
                };
                assert_eq!(outcome.inspections[0], expected_first);
                assert_eq!(outcome.inspections[1], expected_second);
            }
            Ok(outcome) => panic!("Unexpected outcome {:?}", outcome),
        }
    }
}
