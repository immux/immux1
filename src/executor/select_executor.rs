use crate::declarations::commands::{Outcome, SelectCommand, SelectCondition, SelectOutcome};
use crate::declarations::errors::ImmuxResult;
use crate::declarations::instructions::{
    Answer, AtomicGetInstruction, AtomicGetOneInstruction, GetTargetSpec, Instruction,
};
use crate::executor::errors::ExecutorError;
use crate::executor::shared::get_id_list;
use crate::storage::core::{CoreStore, ImmuxDBCore};

pub fn execute_select(select: SelectCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    println!("Executing select {:#?}", select);

    let grouping = &select.grouping;
    let key_list = get_id_list(grouping, core);

    let mut values: Vec<Vec<u8>> = Vec::new();
    for key in key_list {
        println!("reading key {:#?}", key);
        let get_instruction = AtomicGetOneInstruction {
            target: GetTargetSpec { key, height: None },
        };
        match core.execute(&Instruction::AtomicGetOne(get_instruction)) {
            Err(error) => return Err(error),
            Ok(Answer::GetOneOk(answer)) => {
                let value = answer.item.clone();
                println!("Using select.condition {:?}", select.condition);
                let matched = match select.condition {
                    SelectCondition::UnconditionalMatch => true,
                    SelectCondition::JSCode(js_code) => {
                        return Err(ExecutorError::UnimplementedSelectCondition(
                            SelectCondition::JSCode(js_code),
                        )
                        .into())
                    }
                };
                if matched {
                    values.push(value);
                }
            }
            Ok(answer) => return Err(ExecutorError::UnexpectedAnswerType(answer).into()),
        }
    }
    Ok(Outcome::Select(SelectOutcome { values }))
}
