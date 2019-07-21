use crate::declarations::commands::{Command, Outcome};
use crate::declarations::errors::ImmuxResult;

use crate::executor::create_index_executor::execute_create_index;
use crate::executor::insert_executor::execute_insert;
use crate::executor::inspect_executor::execute_inspect;
use crate::executor::name_chain_executor::execute_name_chain;
use crate::executor::pick_chain_executor::execute_pick_chain;
use crate::executor::revert_all_executor::execute_revert_all;
use crate::executor::revert_executor::execute_revert_one;
use crate::executor::select_executor::execute_select;
use crate::storage::core::ImmuxDBCore;

pub fn execute(command: Command, core: &mut ImmuxDBCore) -> ImmuxResult<Outcome> {
    match command {
        Command::PickChain(pick_chain) => execute_pick_chain(pick_chain, core),
        Command::Insert(insert) => execute_insert(insert, core),
        Command::Select(select) => execute_select(select, core),
        Command::NameChain => execute_name_chain(core),
        Command::CreateIndex(create_index) => execute_create_index(create_index, core),
        Command::RevertOne(revert) => execute_revert_one(revert, core),
        Command::RevertAll(revert_all) => execute_revert_all(revert_all, core),
        Command::Inspect(inspect) => execute_inspect(inspect, core),
    }
}

#[cfg(test)]
mod executor_test {
    use crate::declarations::commands::{
        Command, CreateIndexCommand, InsertCommand, InsertCommandSpec, Outcome, PickChainCommand,
        SelectCommand, SelectCondition,
    };
    use crate::declarations::instructions::{Answer, Instruction, ReadNamespaceInstruction};
    use crate::executor::execute::execute;
    use crate::storage::core::{CoreStore, ImmuxDBCore};
    use crate::storage::kv::KeyValueEngine;

    #[test]
    fn test_pick_chain() {
        let default_chain = "default".as_bytes();
        let target_chain = "my little chain".as_bytes();
        let command = Command::PickChain(PickChainCommand {
            new_chain_name: target_chain.to_vec(),
        });
        match ImmuxDBCore::new(&KeyValueEngine::HashMap, default_chain) {
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
            })
            .collect();

        assert!(specs.len() > 0);

        let insert_command = Command::Insert(InsertCommand {
            grouping: grouping.to_vec(),
            targets: specs.clone(),
            insert_with_index: true,
        });
        match ImmuxDBCore::new(&KeyValueEngine::HashMap, default_chain) {
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

    use crate::executor::shared::{ValData};
    
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_create_index() {
        let fixture = vec![
            r#"{"id": 0, "index_field_f64": 0.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
            r#"{"id": 1, "index_field_f64": 2.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
            r#"{"id": 2, "index_field_f64": 2.0, "index_field_string": "string_2", "index_field_bool": true}"#.to_string(),
            r#"{"id": 3, "index_field_f64": 2.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
            r#"{"id": 4, "index_field_f64": 4.0, "index_field_string": "string_4", "index_field_bool": true}"#.to_string(),
            r#"{"id": 5, "index_field_f64": 5.0, "index_field_string": "string_2", "index_field_bool": false}"#.to_string(),
            r#"{"id": 6, "index_field_f64": 7.0, "index_field_string": "string_1", "index_field_bool": false}"#.to_string(),
            r#"{"id": 7, "index_field_f64": 7.0, "index_field_string": "string_1", "index_field_bool": false}"#.to_string(),
            r#"{"id": 8, "index_field_f64": 8.0, "index_field_string": "string_8", "index_field_bool": false}"#.to_string(),
            r#"{"id": 9, "index_field_f64": 8.0, "index_field_string": "string_8", "index_field_bool": false}"#.to_string(),
        ];
        let grouping = "grouping".as_bytes();
        let mut core = ImmuxDBCore::new(&KeyValueEngine::HashMap, "default".as_bytes()).unwrap();
        let index_fields_vec: Vec<Vec<&str>> = vec![
            vec!["index_field_f64"],
            vec!["index_field_string"],
            vec!["index_field_bool"],
            vec!["index_field_bool", "index_field_string", "index_field_f64"],
        ];

        for index_fields in index_fields_vec {
            let index_fields: Vec<Vec<u8>> = index_fields
                .iter()
                .map(|index_field| index_field.to_string().as_bytes().to_vec())
                .collect();
            test_create_index_fields(&fixture, grouping, &index_fields, &mut core);
        }
    }

    fn test_create_index_fields(
        fixture: &Vec<String>,
        grouping: &[u8],
        index_fields: &Vec<Vec<u8>>,
        mut core: &mut ImmuxDBCore,
    ) {
        insert_fixture_into_core(&fixture, grouping, false, &mut core);
        create_indexes_for_grouping(grouping, &mut core, &index_fields);
        assert!(compare_db_index_query_with_filtered_fixture(
            &fixture,
            grouping,
            &index_fields,
            &mut core,
        ));
    }

    fn insert_fixture_into_core(
        fixture: &Vec<String>,
        grouping: &[u8],
        insert_with_index: bool,
        mut core: &mut ImmuxDBCore,
    ) {
        let mut specs = vec![];
        for data in fixture.iter() {
            let js_obj = serde_json::from_str::<Value>(data).unwrap();
            match &js_obj["id".to_string()] {
                Value::Number(number) => {
                    let id = number.as_u64().unwrap();
                    let insert_command_spec = InsertCommandSpec {
                        id: [id as u8].to_vec(),
                        value: data.as_bytes().to_vec(),
                    };
                    specs.push(insert_command_spec);
                }
                _ => {}
            }
        }

        let insert_command = Command::Insert(InsertCommand {
            grouping: grouping.to_vec(),
            targets: specs.clone(),
            insert_with_index,
        });

        match execute(insert_command, &mut core) {
            Err(_error) => panic!("Failed to execute insert command"),
            Ok(Outcome::Insert(_outcome)) => {}
            Ok(_) => panic!("Unexpected outcome type"),
        }
    }

    fn create_indexes_for_grouping(
        grouping: &[u8],
        mut core: &mut ImmuxDBCore,
        index_fields: &Vec<Vec<u8>>,
    ) {
        for index_field in index_fields.clone() {
            let create_index_command = Command::CreateIndex(CreateIndexCommand {
                grouping: grouping.to_vec(),
                field: index_field.to_vec(),
            });

            match execute(create_index_command, &mut core) {
                Err(_error) => panic!("Failed to execute insert command"),
                Ok(Outcome::CreateIndex(_outcome)) => {}
                Ok(_) => panic!("Unexpected outcome type"),
            }
        }
    }

    fn construct_index_field_to_possible_vals_map(
        fixture: &Vec<String>,
        index_fields: &Vec<Vec<u8>>,
    ) -> HashMap<Vec<u8>, Vec<ValData>> {
        let mut res: HashMap<Vec<u8>, Vec<ValData>> = HashMap::new();

        for item in fixture.iter() {
            let js_obj = serde_json::from_str::<Value>(item).unwrap();
            for index_field in index_fields {
                let mut val_data = ValData::Bool(true);
                match &js_obj[&String::from_utf8_lossy(&index_field).into_owned()] {
                    Value::String(string) => {
                        val_data = ValData::String(string.clone());
                    }
                    Value::Bool(boolean) => {
                        val_data = ValData::Bool(*boolean);
                    }
                    Value::Number(number) => {
                        val_data = ValData::Float64(number.as_f64().unwrap());
                    }
                    _ => continue,
                }
                match res.get(&(*index_field).clone()) {
                    Some(value) => {
                        if !value.contains(&val_data) {
                            let mut new_val = value.clone();
                            new_val.push(val_data);
                            res.insert((*index_field).clone(), new_val);
                        }
                    }
                    None => {
                        let value = vec![val_data];
                        res.insert((*index_field).clone(), value);
                    }
                }
            }
        }

        return res;
    }

    fn get_items_from_core_with_index(
        grouping: &[u8],
        index_field: &Vec<u8>,
        val_data: &ValData,
        mut core: &mut ImmuxDBCore,
    ) -> Vec<String> {
        let mut res: Vec<String> = vec![];

        let select_command = SelectCommand {
            grouping: grouping.clone().to_vec(),
            condition: SelectCondition::Kv(index_field.clone(), val_data.clone()),
        };

        match execute(Command::Select(select_command), &mut core) {
            Err(_error) => panic!("Failed to execute select command"),
            Ok(Outcome::Select(select_outcome)) => {
                let values = select_outcome.values;
                for value in values {
                    let json_string = String::from_utf8_lossy(&value).into_owned();
                    res.push(json_string);
                }
            }
            Ok(_) => panic!("Unexpected outcome type"),
        }
        return res;
    }

    fn filter_fixture_with_kv(
        fixture: &Vec<String>,
        index_field: &Vec<u8>,
        val_data: &ValData,
    ) -> Vec<String> {
        let mut res: Vec<String> = vec![];
        for item in fixture.iter() {
            let js_obj = serde_json::from_str::<Value>(item).unwrap();
            match &js_obj[&String::from_utf8_lossy(index_field).into_owned()] {
                Value::String(string) => match val_data {
                    ValData::String(val_data_string) => {
                        if *val_data_string == *string {
                            res.push(item.clone());
                        }
                    }
                    _ => {}
                },
                Value::Bool(boolean) => match val_data {
                    ValData::Bool(val_data_bool) => {
                        if *boolean == *val_data_bool {
                            res.push(item.clone());
                        }
                    }
                    _ => {}
                },
                Value::Number(number) => match val_data {
                    ValData::Float64(val_data_number) => {
                        if number.as_f64().unwrap() == *val_data_number {
                            res.push(item.clone());
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        return res;
    }

    fn compare_db_index_query_with_filtered_fixture(
        fixture: &Vec<String>,
        grouping: &[u8],
        index_fields: &Vec<Vec<u8>>,
        mut core: &mut ImmuxDBCore,
    ) -> bool {
        let index_fields_to_data_vals_map =
            construct_index_field_to_possible_vals_map(fixture, index_fields);
        for (index_field, val_datas) in index_fields_to_data_vals_map.iter() {
            for val_data in val_datas {
                let filtered_fixture = filter_fixture_with_kv(fixture, index_field, val_data);
                let db_index_query_res =
                    get_items_from_core_with_index(grouping, index_field, val_data, &mut core);

                if !two_vecs_are_equal(&db_index_query_res, &filtered_fixture) {
                    return false;
                }
            }
        }
        return true;
    }

    fn two_vecs_are_equal<T>(vec1: &Vec<T>, vec2: &Vec<T>) -> bool
    where
        T: PartialEq,
    {
        if vec1.len() != vec2.len() {
            return false;
        }
        for item in vec1 {
            if !vec2.contains(item) {
                return false;
            }
        }
        for item in vec2 {
            if !vec1.contains(item) {
                return false;
            }
        }
        return true;
    }

    #[test]
    fn test_insert_item_with_index() {
        let mut expected_data = vec![];

        let fixture1 = vec![
                r#"{"id": 0, "index_field_f64": 0.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
                r#"{"id": 1, "index_field_f64": 2.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
                r#"{"id": 2, "index_field_f64": 2.0, "index_field_string": "string_2", "index_field_bool": true}"#.to_string(),
                r#"{"id": 3, "index_field_f64": 2.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
                r#"{"id": 4, "index_field_f64": 4.0, "index_field_string": "string_4", "index_field_bool": true}"#.to_string(),
                r#"{"id": 5, "index_field_f64": 5.0, "index_field_string": "string_2", "index_field_bool": false}"#.to_string(),
                r#"{"id": 6, "index_field_f64": 7.0, "index_field_string": "string_1", "index_field_bool": false}"#.to_string(),
                r#"{"id": 7, "index_field_f64": 7.0, "index_field_string": "string_1", "index_field_bool": false}"#.to_string(),
                r#"{"id": 8, "index_field_f64": 8.0, "index_field_string": "string_8", "index_field_bool": false}"#.to_string(),
                r#"{"id": 9, "index_field_f64": 8.0, "index_field_string": "string_8", "index_field_bool": false}"#.to_string(),
            ];
        let grouping = "grouping".as_bytes();
        let mut core = ImmuxDBCore::new(&KeyValueEngine::HashMap, "default".as_bytes()).unwrap();

        let index_field = "index_field_f64".to_string().as_bytes().to_vec();
        let index_fields = [index_field].to_vec();

        insert_fixture_into_core(&fixture1, grouping, false, &mut core);
        expected_data.extend_from_slice(&fixture1);
        create_indexes_for_grouping(grouping, &mut core, &index_fields);

        let fixture_for_existed_value = vec![
                        r#"{"id": 10, "index_field_f64": 0.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
                        r#"{"id": 11, "index_field_f64": 2.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
                    ];

        insert_fixture_into_core(&fixture_for_existed_value, grouping, true, &mut core);
        expected_data.extend_from_slice(&fixture_for_existed_value);

        assert!(compare_db_index_query_with_filtered_fixture(
            &expected_data,
            grouping,
            &index_fields,
            &mut core,
        ));

        let fixture_for_non_existed_value = vec![
                                r#"{"id": 12, "index_field_f64": 11.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
                                r#"{"id": 13, "index_field_f64": 11.0, "index_field_string": "string_0", "index_field_bool": true}"#.to_string(),
                            ];

        insert_fixture_into_core(&fixture_for_non_existed_value, grouping, true, &mut core);
        expected_data.extend_from_slice(&fixture_for_non_existed_value);

        assert!(compare_db_index_query_with_filtered_fixture(
            &expected_data,
            grouping,
            &index_fields,
            &mut core,
        ));
    }
}
