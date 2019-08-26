use crate::declarations::basics::{StoreKey, UnitContent};
use crate::declarations::commands::{InspectCommand, InspectOutcome, Inspection, Outcome};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::executor::errors::ExecutorError;
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::storage::instructions::{
    Answer, DataAnswer, DataInstruction, DataReadAnswer, DataReadInstruction,
    GetJournalInstruction, Instruction,
};

pub fn execute_inspect(inspect: InspectCommand, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    let store_key = StoreKey::from(inspect.specifier);
    let instruction = GetJournalInstruction { key: store_key };
    match core.execute(&Instruction::Data(DataInstruction::Read(
        DataReadInstruction::GetJournal(instruction),
    ))) {
        Err(error) => return Err(error),
        Ok(answer) => match answer {
            Answer::DataAccess(DataAnswer::Read(DataReadAnswer::GetJournalOk(answer))) => {
                let mut inspections: Vec<Inspection> =
                    Vec::with_capacity(answer.journal.updates.len());
                for update in answer.journal.updates {
                    inspections.push(Inspection {
                        deleted: update.deleted,
                        height: update.height,
                        current_content: UnitContent::parse(update.value.as_bytes())?,
                    });
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
