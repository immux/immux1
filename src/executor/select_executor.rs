use crate::declarations::commands::{Outcome, SelectCommand, SelectCondition, SelectOutcome};
use crate::declarations::errors::ImmuxResult;
use crate::declarations::instructions::{
    Answer, AtomicGetOneInstruction, GetTargetSpec, Instruction,
};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::{get_id_list, get_kv_key};
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::utils;
use crate::utils::utf8_to_string;

pub fn execute_select(select: SelectCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    println!(
        "Executing select on grouping {} for condition {:#?}",
        utf8_to_string(&select.grouping),
        select.condition
    );
    let grouping = &select.grouping;
    let key_list = get_id_list(grouping, core);

    let mut values: Vec<Vec<u8>> = Vec::new();
    for key in key_list {
        println!("reading key {:#?}", utils::utf8_to_string(&key));
        let get_instruction = AtomicGetOneInstruction {
            target: GetTargetSpec {
                key: key.clone(),
                height: None,
            },
        };
        match core.execute(&Instruction::AtomicGetOne(get_instruction)) {
            Err(error) => return Err(error),
            Ok(Answer::GetOneOk(answer)) => {
                let value = answer.item.clone();
                println!("Using select.condition {:?}", select.condition);
                let matched = match &select.condition {
                    SelectCondition::UnconditionalMatch => true,
                    SelectCondition::Id(id) => key == get_kv_key(&select.grouping, id),
                    SelectCondition::JSCode(js_code) => {
                        return Err(ExecutorError::UnimplementedSelectCondition(
                            SelectCondition::JSCode(js_code.to_owned()),
                        )
                        .into())
                    }
                };
                if matched {
                    let value = answer.item;
                    values.push(value);
                }
            }
            Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
        }
    }
    Ok(Outcome::Select(SelectOutcome { values }))
}
