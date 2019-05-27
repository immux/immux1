use crate::declarations::commands::{Command, Outcome, SelectCondition};
use crate::declarations::errors::UnumResult;
use crate::declarations::instructions::Answer;
use crate::executor::insert_executor::execute_insert;
use crate::executor::pick_chain_executor::execute_pick_chain;
use crate::executor::select_executor::execute_select;
use crate::storage::core::UnumCore;

#[derive(Debug)]
pub enum ExecutorError {
    UnexpectedAnswerType(Answer),
    UnimplementedSelectCondition(SelectCondition),
}

pub fn execute(command: Command, core: &mut UnumCore) -> UnumResult<Outcome> {
    match command {
        Command::PickChain(pick_chain) => execute_pick_chain(pick_chain, core),
        Command::Insert(insert) => execute_insert(insert, core),
        Command::Select(select) => execute_select(select, core),
    }
}

#[cfg(test)]
mod executor_test {
    use crate::declarations::commands::{
        Command, InsertCommand, InsertCommandSpec, Outcome, PickChainCommand, SelectCommand,
        SelectCondition,
    };
    use crate::declarations::instructions::{Answer, Instruction, ReadNamespaceInstruction};
    use crate::executor::execute::execute;
    use crate::storage::core::{CoreStore, UnumCore};
    use crate::storage::kv::KeyValueEngine;

    #[test]
    fn test_pick_chain() {
        let default_chain = "default".as_bytes();
        let target_chain = "my little chain".as_bytes();
        let command = Command::PickChain(PickChainCommand {
            new_chain_name: target_chain.to_vec(),
        });
        match UnumCore::new(&KeyValueEngine::HashMap, default_chain) {
            Err(_error) => panic!("Cannot initialized core"),
            Ok(mut core) => match execute(command, &mut core) {
                Err(_error) => panic!("Failed to execute pick chain command"),
                Ok(Outcome::PickChain(outcome)) => {
                    assert_eq!(outcome.new_chain_name, target_chain);

                    let instruction = ReadNamespaceInstruction {};
                    match core.execute(&Instruction::ReadNamespace(instruction)) {
                        Err(_error) => panic!("Cannot read namespace"),
                        Ok(Answer::ReadNamespaceOk(answer)) => {
                            // Inspect the actual namespace
                            assert_eq!(answer.namespace, target_chain);
                        }
                        Ok(_) => panic!("Unexpected answer"),
                    }
                }
                Ok(_) => panic!("Unexpected outcome type"),
            },
        }
    }

    #[test]
    fn test_simple_insert_select() {
        let default_chain = "default".as_bytes();
        let grouping = "grouping".as_bytes();

        let specs: Vec<InsertCommandSpec> = (1..100)
            .collect::<Vec<u8>>()
            .iter()
            .map(|datum| InsertCommandSpec {
                id: vec![1, 2, 3, *datum],
                value: vec![1, 2, 3, *datum],
                grouping: grouping.to_vec(),
            })
            .collect();

        assert!(specs.len() > 0);

        let insert_command = Command::Insert(InsertCommand {
            targets: specs.clone(),
        });
        match UnumCore::new(&KeyValueEngine::HashMap, default_chain) {
            Err(_error) => panic!("Cannot initialized core"),
            Ok(mut core) => match execute(insert_command, &mut core) {
                Err(_error) => panic!("Failed to execute insert command"),
                Ok(Outcome::Insert(outcome)) => {
                    assert_eq!(outcome.count, specs.len());
                    let select_command = Command::Select(SelectCommand {
                        grouping: grouping.to_vec(),
                        condition: SelectCondition::UnconditionalMatch,
                    });
                    match execute(select_command, &mut core) {
                        Err(_error) => panic!("Failed to execute select command"),
                        Ok(Outcome::Select(outcome)) => {
                            assert_eq!(outcome.values.len(), specs.len());
                            for spec in specs.iter() {
                                assert!(outcome.values.contains(&spec.value))
                            }
                        }
                        Ok(_) => panic!("Unexpected outcome type"),
                    }
                }
                Ok(_) => panic!("Unexpected outcome type"),
            },
        }
    }
}
