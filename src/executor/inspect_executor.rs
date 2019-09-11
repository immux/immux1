use crate::declarations::basics::{StoreKey, UnitContent};
use crate::declarations::commands::{InspectCommand, InspectOutcome, Inspection, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DataAnswer, DataReadAnswer, GetJournalInstruction, GetOneInstruction, Instruction,
};

pub fn execute_inspect(inspect: InspectCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
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
                                    content: Some(UnitContent::parse(data)?),
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
