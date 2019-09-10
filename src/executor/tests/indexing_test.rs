#[cfg(test)]
mod indexing_test {
    use serde_json::Value as JsonValue;
    use std::vec::IntoIter as VecIntoIter;

    use crate::config::DEFAULT_PERMANENCE_PATH;
    use crate::declarations::basics::{
        GroupingLabel, NameProperty, PropertyName, PropertyNameList, UnitContent, UnitId,
    };
    use crate::declarations::commands::{
        Command, CreateIndexCommand, InsertCommand, InsertCommandSpec, Outcome, SelectCommand,
        SelectCondition,
    };

    use crate::executor::execute::execute;
    use crate::executor::reverse_index::ReverseIndex;
    use crate::storage::core::ImmuxDBCore;
    use crate::storage::instructions::StoreNamespace;
    use crate::storage::kv::KeyValueEngine;
    use immuxdb_dev_utils::reset_db_dir;

    type JsonTableRow = (UnitId, String);

    #[derive(Clone)]
    struct JsonTable {
        inner: Vec<JsonTableRow>,
    }

    impl JsonTable {
        pub fn load_with_auto_id(initial_id: UnitId, data: &[&str]) -> Self {
            let table = data
                .iter()
                .enumerate()
                .map(|x| {
                    (
                        UnitId::new(initial_id.as_int() + x.0 as u128),
                        x.1.to_string(),
                    )
                })
                .collect();
            return JsonTable { inner: table };
        }
        pub fn get_inner(&self) -> &[(UnitId, String)] {
            &self.inner
        }
        pub fn filter_rows_by_property(&self, name_property: &NameProperty) -> Vec<JsonTableRow> {
            let (name, property) = name_property;
            return self
                .inner
                .iter()
                .filter(|row| {
                    let json_value = serde_json::from_str::<JsonValue>(&row.1).unwrap();
                    let name_str = name.to_string();
                    match json_value.get(name_str) {
                        None => return false,
                        Some(json_property) => match &json_property {
                            JsonValue::String(string) => match property {
                                UnitContent::String(property_str) => {
                                    return *string == *property_str
                                }
                                _ => return false,
                            },
                            JsonValue::Bool(boolean) => match property {
                                UnitContent::Bool(property_bool) => {
                                    return *boolean == *property_bool
                                }
                                _ => return false,
                            },
                            JsonValue::Number(number) => match property {
                                UnitContent::Float64(property_f64) => {
                                    return number.as_f64().unwrap() == *property_f64;
                                }
                                _ => return false,
                            },
                            JsonValue::Null => match property {
                                UnitContent::Nil => return true,
                                _ => return false,
                            },
                            _ => return false,
                        },
                    }
                })
                .map(|np| np.to_owned())
                .collect();
        }
        pub fn merge(&mut self, new_table: &Self) {
            for row in new_table.inner.iter() {
                self.inner.push((row.0, row.1.clone()))
            }
        }
        pub fn size(&self) -> usize {
            self.inner.len()
        }
    }

    impl IntoIterator for JsonTable {
        type Item = JsonTableRow;
        type IntoIter = VecIntoIter<JsonTableRow>;

        fn into_iter(self) -> Self::IntoIter {
            self.inner.into_iter()
        }
    }

    fn get_initial_data() -> JsonTable {
        JsonTable::load_with_auto_id(
            UnitId::new(0),
            &[
                // Regular data
                r#"{"f64": 1.0, "str": "A", "bool": true}"#,
                r#"{"f64": 2.0, "str": "B", "bool": true}"#,
                r#"{"f64": 2.0, "str": "C", "bool": true}"#,
                r#"{"f64": 2.0, "str": "C", "bool": true}"#,
                r#"{"f64": 5.0, "str": "B", "bool": false}"#,
                r#"{"f64": 2.1, "str": "X", "bool": true}"#,
                r#"{"f64": 7.0, "str": "B", "bool": false}"#,
                r#"{"f64": 2.1, "str": "D", "bool": true}"#,
                r#"{"f64": 4.0, "str": "X", "bool": true}"#,
                r#"{"f64": 2.0, "str": "C", "bool": true}"#,
                r#"{"f64": 7.0, "str": "C", "bool": false}"#,
            ],
        )
    }

    fn reset_core(label: &str) -> ImmuxDBCore {
        let data_path = DEFAULT_PERMANENCE_PATH;
        reset_db_dir(&format!("{}{}", data_path, label)).unwrap();
        ImmuxDBCore::new(
            &KeyValueEngine::Rocks,
            data_path,
            &StoreNamespace::new(label.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn test_retrieval_by_index() {
        let mut core = reset_core("test_create_index_completeness");
        let name_groups: Vec<Vec<&str>> = vec![
            vec!["f64"],
            vec!["str"],
            vec!["bool"],
            vec!["bool", "str", "f64"],
        ];

        let fixture = get_initial_data();

        for names in name_groups {
            let grouping = GroupingLabel::from(names.join("-").as_bytes());
            let name_list = PropertyNameList::new(
                names
                    .into_iter()
                    .map(|name_str| PropertyName::new(name_str.as_bytes()))
                    .collect(),
            );

            insert_table_to_db(&fixture, &grouping, &mut core);
            create_indices_for_grouping(&grouping, &mut core, &name_list);

            verify_db_consistency_against_memory_table(&mut core, &grouping, &name_list, &fixture);
        }
    }

    fn insert_table_to_db(table: &JsonTable, grouping: &GroupingLabel, mut core: &mut ImmuxDBCore) {
        let specs: Vec<InsertCommandSpec> = table
            .clone()
            .into_iter()
            .map(|x| InsertCommandSpec {
                id: x.0 as UnitId,
                content: UnitContent::JsonString(x.1),
            })
            .collect();

        let insert_command = Command::Insert(InsertCommand {
            grouping: grouping.to_owned(),
            targets: specs,
        });

        match execute(insert_command, &mut core) {
            Err(error) => panic!("Failed to execute insert command {:x?}", error),
            Ok(Outcome::Insert(_outcome)) => {}
            Ok(_) => panic!("Unexpected outcome type"),
        }
    }

    fn create_indices_for_grouping(
        grouping: &GroupingLabel,
        mut core: &mut ImmuxDBCore,
        names: &PropertyNameList,
    ) {
        for name in names.clone() {
            let create_index_command = Command::CreateIndex(CreateIndexCommand {
                grouping: grouping.to_owned(),
                name,
            });

            match execute(create_index_command, &mut core) {
                Err(error) => panic!("Failed to execute insert command {:x?}", error),
                Ok(Outcome::CreateIndex(_outcome)) => {}
                Ok(_) => panic!("Unexpected outcome type"),
            }
        }
    }

    fn filter_db_with_name_property(
        core: &mut ImmuxDBCore,
        grouping: &GroupingLabel,
        name_property: &NameProperty,
    ) -> Vec<(UnitId, String)> {
        let (name, content) = name_property;
        let select_by_name_property = Command::Select(SelectCommand {
            grouping: grouping.to_owned(),
            condition: SelectCondition::NameProperty(name.to_owned(), content.to_owned()),
        });

        match execute(select_by_name_property, core) {
            Err(error) => panic!("Failed to execute select command: {:x?}", error),
            Ok(Outcome::Select(select_outcome)) => {
                return select_outcome
                    .units
                    .into_iter()
                    .map(|unit| match &unit.content {
                        UnitContent::JsonString(s) => (unit.id, s.to_owned()),
                        _ => panic!("ERROR: Unexpected unit content type"),
                    })
                    .collect();
            }
            Ok(_) => panic!("Unexpected outcome type"),
        }
    }

    fn verify_db_consistency_against_memory_table(
        mut core: &mut ImmuxDBCore,
        grouping: &GroupingLabel,
        indexed_names: &PropertyNameList,
        table: &JsonTable,
    ) -> () {
        let reverse_index = ReverseIndex::from_jsons(table.get_inner(), indexed_names).unwrap();
        for ((name, property_bytes), _ids) in reverse_index.into_iter() {
            let content = UnitContent::parse(&property_bytes).unwrap();
            let units_from_db =
                filter_db_with_name_property(&mut core, grouping, &(name.clone(), content.clone()));
            let units_from_memory_table =
                table.filter_rows_by_property(&(name.clone(), content.clone()));
            assert!(are_vecs_equal(&units_from_db, &units_from_memory_table))
        }
    }

    fn are_vecs_equal<T>(vec1: &[T], vec2: &[T]) -> bool
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
        let grouping = GroupingLabel::from("grouping".as_bytes());

        let mut core = reset_core("test_insert_item_with_index");

        let indexed_name_list = PropertyNameList::new(vec![PropertyName::new("f64".as_bytes())]);

        // Initialization
        let mut simulated_state = get_initial_data();
        insert_table_to_db(&simulated_state, &grouping, &mut core);
        create_indices_for_grouping(&grouping, &mut core, &indexed_name_list);

        let mut insert_then_verify = |new_jsons: &[&str]| {
            let initial_id = UnitId::new(simulated_state.size() as u128);
            let new_data = JsonTable::load_with_auto_id(initial_id, new_jsons);
            simulated_state.merge(&new_data);
            insert_table_to_db(&new_data, &grouping, &mut core);
            verify_db_consistency_against_memory_table(
                &mut core,
                &grouping,
                &indexed_name_list,
                &simulated_state,
            );
        };

        // Add to indexed name with known properties
        insert_then_verify(&[
            r#"{"f64": 0.0, "str": "C", "bool": true}"#,
            r#"{"f64": 2.0, "str": "E", "bool": true}"#,
        ]);

        // Add to indexed name with more challenging items
        insert_then_verify(&[
            r#"{"f64": 1.5, "str": "E", "bool": true}"#, // normal reference data
            r#"{}"#,                                     // empty object
            r#"{"f64": -10000}"#,                        // "missing" fields
            r#"{"str": null, "f64": null}"#,             // nulls
            r#"{"str": null, "another_field": null}"#,   // nulls in unseen fields
            r#"{"new_key": 0}"#,                         // previously unseen keys
            r#"{"new_key": 1}"#,                         // previously unseen keys
            r#"{"new_key": null}"#,                      // previously unseen keys
            r#"{"new_key": null}"#,                      // previously unseen keys
            r#"{"bool": true, "f64": -1, "str": "OK"}"#, // shifted order
            r#"{"f64": "ss", "str": false, "bool": 1.0}"#, // keys containing various types
            r#"{"f64": true, "str": 3, "bool": "ok"}"#,  // keys containing various types
            r#"{"f64": "ss", "str": false, "bool": 1.0}"#, // keys containing various types
        ]);
    }

}
