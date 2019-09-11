mod indexing_test;

#[cfg(test)]
mod executor_test {
    use crate::config::DEFAULT_PERMANENCE_PATH;
    use crate::declarations::basics::{ChainName, GroupingLabel, Unit, UnitContent, UnitId};
    use crate::declarations::commands::{
        Command, InsertCommand, InsertCommandSpec, Outcome, PickChainCommand, SelectCommand,
        SelectCondition,
    };
    use crate::executor::execute::execute;
    use crate::storage::core::{CoreStore, ImmuxDBCore};
    use crate::storage::instructions::{
        Answer, DBSystemAnswer, DBSystemInstruction, Instruction, ReadNamespaceInstruction,
        StoreNamespace,
    };
    use crate::storage::kv::KeyValueEngine;
    use immuxdb_dev_utils::reset_db_dir;

    #[test]
    fn test_pick_chain() {
        let default_chain = ChainName::from("default");
        let target_chain = ChainName::from("my little chain");
        let pick_chain = Command::PickChain(PickChainCommand {
            new_chain_name: target_chain.clone(),
        });
        let default_namespace = StoreNamespace::from(default_chain);
        match ImmuxDBCore::new(
            &KeyValueEngine::HashMap,
            DEFAULT_PERMANENCE_PATH,
            &default_namespace,
        ) {
            Err(_error) => panic!("Cannot initialized core"),
            Ok(mut core) => match execute(pick_chain, &mut core) {
                Err(_error) => panic!("Failed to execute pick chain command"),
                Ok(Outcome::PickChain(outcome)) => {
                    assert_eq!(outcome.new_chain_name, target_chain);

                    let read_namespace = Instruction::DBSystem(DBSystemInstruction::ReadNamespace(
                        ReadNamespaceInstruction {},
                    ));
                    match core.execute(&read_namespace) {
                        Err(_error) => panic!("Cannot read namespace"),
                        Ok(Answer::DBSystem(DBSystemAnswer::ReadNamespaceOk(answer))) => {
                            // Inspect the actual namespace
                            assert_eq!(answer.namespace.as_bytes(), target_chain.as_bytes());
                        }
                        Ok(_) => panic!("Unexpected answer"),
                    }
                }
                Ok(_) => panic!("Unexpected outcome type"),
            },
        }
    }

    /// Insert some simple data and get them back.
    /// Inserts and Selects need to be tested together.
    #[test]
    fn test_insert_and_select_by_unconditional_match() {
        let data_root = format!("/tmp/immuxdb_test/");
        reset_db_dir(&data_root).unwrap();

        let namespace = StoreNamespace::new("default".as_bytes());
        let grouping = GroupingLabel::from("grouping".as_bytes());

        let specs: Vec<InsertCommandSpec> = (1..5)
            .map(|i| InsertCommandSpec {
                id: UnitId::new(i as u128),
                content: UnitContent::Bytes(vec![1, 2, 3, i as u8]),
            })
            .collect();

        assert!(specs.len() > 0);

        let insert_command = Command::Insert(InsertCommand {
            grouping: grouping.clone(),
            targets: specs.clone(),
        });
        match ImmuxDBCore::new(&KeyValueEngine::Rocks, &data_root, &namespace) {
            Err(_error) => panic!("Cannot initialized core"),
            Ok(mut core) => match execute(insert_command, &mut core) {
                Err(error) => panic!("Failed to execute insert command: {:x?}", error),
                Ok(Outcome::Insert(outcome)) => {
                    assert_eq!(outcome.count, specs.len());
                    let select_command = Command::Select(SelectCommand {
                        grouping,
                        condition: SelectCondition::UnconditionalMatch,
                    });
                    match execute(select_command, &mut core) {
                        Err(_error) => panic!("Failed to execute select command"),
                        Ok(Outcome::Select(outcome)) => {
                            assert_eq!(outcome.units.len(), specs.len());
                            for spec in specs.iter() {
                                let unit = Unit {
                                    id: spec.id,
                                    content: spec.content.clone(),
                                };
                                assert!(outcome.units.contains(&unit))
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
