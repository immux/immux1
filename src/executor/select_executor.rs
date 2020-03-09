use std::convert::TryFrom;

use serde_json::Value as JsonValue;

use crate::declarations::basics::{
    GroupingLabel, IdList, StoreKey, StoreKeyFragment, Unit, UnitContent, UnitSpecifier,
};
use crate::declarations::commands::{Outcome, SelectCommand, SelectCondition, SelectOutcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::get_store_key_of_indexed_id_list;
use crate::storage::core::CoreStore;
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction, GetManyInstruction,
    GetManyTargetSpec, GetOneInstruction, Instruction,
};
use crate::storage::vkv::VkvError;

fn get_all_in_grouping(
    grouping: &GroupingLabel,
    core: &mut impl CoreStore,
) -> ImmuxResult<Vec<Unit>> {
    let prefix: StoreKeyFragment = grouping.marshal().into();
    let get_all = Instruction::DataAccess(DataInstruction::Read(DataReadInstruction::GetMany(
        GetManyInstruction {
            height: None,
            targets: GetManyTargetSpec::KeyPrefix(prefix),
        },
    )));
    match core.execute(&get_all) {
        Err(error) => return Err(error),
        Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetManyOk(answer)))) => {
            let mut units = Vec::with_capacity(answer.data.len());
            // Remove None values (they're none because they are removed)
            let extant_data = answer
                .data
                .iter()
                .filter_map(|pair| pair.1.inner().as_ref().map(|data| (pair.0.clone(), data)));
            for pair in extant_data {
                let (boxed_store_key, store_value) = pair;
                let store_key = StoreKey::new(boxed_store_key.as_slice());
                let specifier = UnitSpecifier::try_from(store_key)?;
                let id = specifier.get_id();
                let content = UnitContent::parse_data(&store_value)?;
                units.push(Unit { id, content })
            }
            return Ok(units);
        }
        Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
    }
}

pub fn execute_select(select: SelectCommand, core: &mut impl CoreStore) -> ImmuxResult<Outcome> {
    match &select.condition {
        SelectCondition::UnconditionalMatch => get_all_in_grouping(&select.grouping, core)
            .map(|units| Ok(Outcome::Select(SelectOutcome { units })))?,
        SelectCondition::Id(id) => {
            let key = StoreKey::build(&select.grouping, id.to_owned());
            let instruction = Instruction::DataAccess(DataInstruction::Read(
                DataReadInstruction::GetOne(GetOneInstruction { key, height: None }),
            ));
            match core.execute(&instruction) {
                Err(error) => return Err(error),
                Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                    match answer.value.inner() {
                        None => Err(ExecutorError::CannotFindId(*id).into()),
                        Some(data) => {
                            let content = UnitContent::parse_data(data)?;
                            let units = vec![Unit { id: *id, content }];
                            Ok(Outcome::Select(SelectOutcome { units }))
                        }
                    }
                }
                Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
            }
        }
        SelectCondition::NameProperty(name, property) => {
            let grouping = &select.grouping;

            let units: Vec<Unit> = {
                let mut result: Vec<Unit> = Vec::new();
                let get_indexed_id_list = Instruction::DataAccess(DataInstruction::Read(
                    DataReadInstruction::GetOne(GetOneInstruction {
                        key: get_store_key_of_indexed_id_list(grouping, name, property),
                        height: None,
                    }),
                ));

                match core.execute(&get_indexed_id_list) {
                    Err(ImmuxError::VKV(VkvError::MissingJournal(_error))) => {
                        // No index for the name-property
                        let all_units = get_all_in_grouping(&select.grouping, core)?;
                        let proper_units =
                            all_units.into_iter().filter(|unit| match &unit.content {
                                UnitContent::JsonString(s) => {
                                    match serde_json::from_str::<JsonValue>(&s) {
                                        Err(_) => return false,
                                        Ok(json) => {
                                            let key = name.to_string();
                                            match json.get(key) {
                                                None => return false,
                                                Some(value) => return property == value,
                                            }
                                        }
                                    }
                                }
                                _ => false,
                            });
                        result = proper_units.collect();
                    }
                    Err(error) => {
                        return Err(error.into());
                    }
                    Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                        match answer.value.inner() {
                            None => return Err(ExecutorError::NoneReverseIndex.into()),
                            Some(data) => {
                                let id_list = IdList::try_from(data.as_slice())?;
                                for id in id_list {
                                    let get_data = Instruction::DataAccess(DataInstruction::Read(
                                        DataReadInstruction::GetOne(GetOneInstruction {
                                            key: StoreKey::build(grouping, id),
                                            height: None,
                                        }),
                                    ));

                                    match core.execute(&get_data) {
                                        Err(error) => return Err(error),
                                        Ok(Answer::DataAccess(DataAnswer::Read(
                                            DataReadAnswer::GetOneOk(answer),
                                        ))) => {
                                            let _content = match answer.value.inner() {
                                                None => (),
                                                Some(data) => {
                                                    let content = UnitContent::parse_data(data)?;
                                                    let unit = Unit { id, content };
                                                    result.push(unit);
                                                }
                                            };
                                        }
                                        Ok(answer) => {
                                            return Err(ExecutorError::UnexpectedAnswerType(
                                                answer,
                                            )
                                            .into());
                                        }
                                    };
                                }
                            }
                        }
                    }
                    Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
                };
                result
            };

            Ok(Outcome::Select(SelectOutcome { units }))
        }
        SelectCondition::JSCode(js_code) => {
            return Err(
                ExecutorError::UnimplementedSelectCondition(SelectCondition::JSCode(
                    js_code.to_owned(),
                ))
                .into(),
            );
        }
    }
}
