use std::convert::TryFrom;

use crate::declarations::basics::{
    IdList, StoreKey, StoreKeyFragment, Unit, UnitContent, UnitSpecifier,
};
use crate::declarations::commands::{Outcome, SelectCommand, SelectCondition, SelectOutcome};
use crate::declarations::errors::ImmuxResult;
use crate::executor::errors::ExecutorError;
use crate::executor::shared::get_store_key_of_indexed_id_list;
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction, GetManyInstruction,
    GetManyTargetSpec, GetOneInstruction, Instruction,
};

pub fn execute_select(select: SelectCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    match &select.condition {
        SelectCondition::UnconditionalMatch => {
            let grouping_bytes: Vec<u8> = select.grouping.marshal();
            let prefix: StoreKeyFragment = grouping_bytes.into();
            let get_all = Instruction::Data(DataInstruction::Read(DataReadInstruction::GetMany(
                GetManyInstruction {
                    height: None,
                    targets: GetManyTargetSpec::KeyPrefix(prefix),
                },
            )));
            match core.execute(&get_all) {
                Err(error) => return Err(error),
                Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetManyOk(answer)))) => {
                    let mut units = Vec::with_capacity(answer.data.len());
                    for pair in answer.data {
                        let (boxed_store_key, store_value) = pair;
                        let store_key = StoreKey::new(boxed_store_key.as_slice());
                        let specifier = UnitSpecifier::try_from(store_key)?;
                        let id = specifier.get_id();
                        let content = UnitContent::parse(&store_value.inner())?;
                        units.push(Unit { id, content })
                    }
                    return Ok(Outcome::Select(SelectOutcome { units }));
                }
                Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
            }
        }
        SelectCondition::Id(id) => {
            let mut units: Vec<Unit> = Vec::new();
            let key = StoreKey::build(&select.grouping, id.to_owned());
            let instruction = Instruction::Data(DataInstruction::Read(
                DataReadInstruction::GetOne(GetOneInstruction { key, height: None }),
            ));
            match core.execute(&instruction) {
                Err(error) => return Err(error),
                Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                    let content = UnitContent::parse(&answer.value.as_bytes())?;
                    units.push(Unit { id: *id, content });
                    Ok(Outcome::Select(SelectOutcome { units }))
                }
                Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
            }
        }
        SelectCondition::NameProperty(name, property) => {
            let grouping = &select.grouping;

            let units: Vec<Unit> = {
                let mut result: Vec<Unit> = Vec::new();
                let get_indexed_id_list = Instruction::Data(DataInstruction::Read(
                    DataReadInstruction::GetOne(GetOneInstruction {
                        key: get_store_key_of_indexed_id_list(grouping, name, property),
                        height: None,
                    }),
                ));
                match core.execute(&get_indexed_id_list) {
                    Err(error) => {
                        return Err(error.into());
                    }
                    Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetOneOk(answer)))) => {
                        let id_list_bytes: Vec<u8> = answer.value.into();
                        let id_list = IdList::try_from(id_list_bytes.as_slice())?;
                        for id in id_list {
                            let get_data = Instruction::Data(DataInstruction::Read(
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
                                    let content = UnitContent::parse(&answer.value.as_bytes())?;
                                    let unit = Unit { id, content };
                                    result.push(unit);
                                }
                                Ok(answer) => {
                                    return Err(ExecutorError::UnexpectedAnswerType(answer).into());
                                }
                            };
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
