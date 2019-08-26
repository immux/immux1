use std::convert::TryFrom;

use serde_json::Value as JsonValue;

use crate::declarations::basics::{
    StoreKey, StoreKeyFragment, StoreValue, UnitContent, UnitSpecifier,
};
use crate::declarations::commands::{CreateIndexCommand, CreateIndexOutcome, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::executor::reverse_index::ReverseIndex;
use crate::executor::shared::{
    get_indexed_names_list_with_fallback, get_store_key_of_indexed_id_list, set_indexed_names_list,
};
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction, DataWriteAnswer,
    DataWriteInstruction, GetManyInstruction, GetManyTargetSpec, Instruction, SetManyInstruction,
    SetTargetSpec,
};

pub fn execute_create_index(
    command: CreateIndexCommand,
    core: &mut ImmuxDBCore,
) -> ImmuxResult<Outcome> {
    let grouping = &command.grouping;
    let mut indexed_names = get_indexed_names_list_with_fallback(grouping, core)?;
    indexed_names.add(command.name.clone());
    set_indexed_names_list(grouping, &indexed_names, core)?;

    let reverse_index: ReverseIndex = {
        let mut index = ReverseIndex::new();

        let prefix: StoreKeyFragment = grouping.marshal().into();
        let get_by_prefix = Instruction::Data(DataInstruction::Read(DataReadInstruction::GetMany(
            GetManyInstruction {
                height: None,
                targets: GetManyTargetSpec::KeyPrefix(prefix),
            },
        )));

        match core.execute(&get_by_prefix) {
            Err(error) => return Err(error),
            Ok(Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetManyOk(answer)))) => {
                for (store_key, store_value) in answer.data.into_iter() {
                    let content = UnitContent::parse(store_value.inner())?;
                    match content {
                        UnitContent::JsonString(json_string) => {
                            match serde_json::from_str::<JsonValue>(&json_string) {
                                Err(_error) => {
                                    println!("error!");
                                    continue;
                                }
                                Ok(json) => {
                                    let unboxed_key: StoreKey = store_key.into();
                                    let specifier = UnitSpecifier::try_from(unboxed_key)?;
                                    let id = specifier.get_id();
                                    index.index_new_json(id, &json, &command.name)?;
                                }
                            }
                        }
                        _ => {
                            println!("Skipping unexpected shape!");
                        }
                    }
                }
            }
            Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
        }
        index
    };

    let targets = {
        let mut targets = Vec::new();
        for ((name, property_bytes), ids) in reverse_index {
            let property = UnitContent::parse(&property_bytes)?;
            let key = get_store_key_of_indexed_id_list(grouping, &name, &property);
            let value = StoreValue::new(ids.into());
            let insert_command_spec = SetTargetSpec { key, value };
            targets.push(insert_command_spec);
        }
        targets
    };

    let instruction: Instruction = Instruction::Data(DataInstruction::Write(
        DataWriteInstruction::SetMany(SetManyInstruction { targets }),
    ));

    match core.execute(&instruction) {
        Err(error) => return Err(error),
        Ok(Answer::DataAccess(DataAnswer::Write(DataWriteAnswer::SetOk(_answer)))) => {
            return Ok(Outcome::CreateIndex(CreateIndexOutcome {}));
        }
        Ok(answer) => {
            return Err(ImmuxError::Executor(ExecutorError::UnexpectedAnswerType(
                answer,
            )));
        }
    }
}
