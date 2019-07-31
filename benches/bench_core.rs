pub mod executor_benchmark {

    use immuxdb::declarations::commands::{
        Command, CreateIndexCommand, InsertCommand, InsertCommandSpec, Outcome, SelectCommand,
        SelectCondition,
    };
    use immuxdb::executor::execute::execute;
    use immuxdb::executor::shared::ValData;
    use immuxdb::storage::core::{CoreStore, ImmuxDBCore};
    use immuxdb::storage::kv::KeyValueEngine;
    use immuxdb::utils::u32_to_u8_array;
    use rand::Rng;
    use std::ops::Range;
    use std::thread::sleep;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    static GROUPING: &str = "grouping";
    static NAMESPACE: &str = "default";
    static INDEX_FIELD: &str = "age";
    static MINIMUM_AGE: u8 = 20;
    static MAX_AGE: u8 = 50;

    fn get_random_age() -> u8 {
        let mut rng = rand::thread_rng();
        let age = rng.gen_range(MINIMUM_AGE, MAX_AGE);
        return age;
    }

    fn generate_random_json_string() -> String {
        let boilerplate = format!(
            r#"{{ "name":"Tom", "age":{}, "sex":"male", "password": "1234" }}"#,
            get_random_age(),
        );

        return boilerplate;
    }

    fn get_multi_jsons_insert_commands(
        id_range: Range<u32>,
        insert_with_index: bool,
    ) -> Vec<Command> {
        let mut insert_commands = vec![];
        for id in id_range {
            let insert_command_spec = InsertCommandSpec {
                id: u32_to_u8_array(id).to_vec(),
                value: generate_random_json_string().as_bytes().to_vec(),
            };
            let command = Command::Insert(InsertCommand {
                grouping: GROUPING.as_bytes().to_vec(),
                targets: vec![insert_command_spec],
                insert_with_index,
            });
            insert_commands.push(command);
        }
        return insert_commands;
    }

    fn get_batch_json_insert_command(id_range: Range<u32>, insert_with_index: bool) -> Command {
        let mut insert_command_specs = vec![];
        for id in id_range {
            let insert_command_spec = InsertCommandSpec {
                id: u32_to_u8_array(id).to_vec(),
                value: generate_random_json_string().as_bytes().to_vec(),
            };
            insert_command_specs.push(insert_command_spec);
        }
        let command = Command::Insert(InsertCommand {
            grouping: GROUPING.as_bytes().to_vec(),
            targets: insert_command_specs,
            insert_with_index,
        });
        return command;
    }

    fn get_multi_select_command_by_ids(id_range: Range<u32>) -> Vec<Command> {
        let mut select_commands = vec![];
        for id in id_range {
            let select_condition = SelectCondition::Id(u32_to_u8_array(id).to_vec());
            let select_command = SelectCommand {
                grouping: GROUPING.as_bytes().to_vec(),
                condition: select_condition,
            };
            let command = Command::Select(select_command);
            select_commands.push(command);
        }
        return select_commands;
    }

    fn get_select_command_by_kv(key: Vec<u8>, val: ValData) -> Command {
        let select_condition = SelectCondition::Kv(key, val);
        let select_command = SelectCommand {
            grouping: GROUPING.as_bytes().to_vec(),
            condition: select_condition,
        };
        let command = Command::Select(select_command);
        return command;
    }

    fn get_create_index_command(index_field: &str) -> Command {
        let create_index_command = CreateIndexCommand {
            grouping: GROUPING.to_string().as_bytes().to_vec(),
            field: index_field.to_string().as_bytes().to_vec(),
        };
        return Command::CreateIndex(create_index_command);
    }

    pub fn benchmark_single_insert() {
        let id_range = (0..100);
        let mut core = ImmuxDBCore::new(&KeyValueEngine::Rocks, NAMESPACE.as_bytes()).unwrap();
        let command = get_batch_json_insert_command(id_range, false);
        execute(command, &mut core).unwrap();
    }

    pub fn benchmark_multi_insert() {
        let id_range = (1..100);
        let mut core = ImmuxDBCore::new(&KeyValueEngine::Rocks, NAMESPACE.as_bytes()).unwrap();
        let commands = get_multi_jsons_insert_commands(id_range, false);
        for command in commands {
            execute(command, &mut core).unwrap();
        }
    }

    fn benchmark_single_select() {
        let id_range = (0..1000);
        let mut core = ImmuxDBCore::new(&KeyValueEngine::Rocks, NAMESPACE.as_bytes()).unwrap();
        let insert_command = get_batch_json_insert_command(id_range.clone(), false);
        execute(insert_command, &mut core).unwrap();

        let select_commands = get_multi_select_command_by_ids(id_range);
        for command in select_commands {
            execute(command, &mut core).unwrap();
        }
    }

    fn benchmark_create_index() {
        let id_range = (0..1000);
        let mut core = ImmuxDBCore::new(&KeyValueEngine::Rocks, NAMESPACE.as_bytes()).unwrap();
        let insert_command = get_batch_json_insert_command(id_range.clone(), false);
        execute(insert_command, &mut core).unwrap();

        let create_index_command = get_create_index_command(INDEX_FIELD);
        execute(create_index_command, &mut core).unwrap();
    }

    fn benchmark_select_by_index() {
        let id_range = (0..1000);
        let mut core = ImmuxDBCore::new(&KeyValueEngine::Rocks, NAMESPACE.as_bytes()).unwrap();
        let insert_command = get_batch_json_insert_command(id_range.clone(), false);
        execute(insert_command, &mut core).unwrap();

        let create_index_command = get_create_index_command(INDEX_FIELD);
        execute(create_index_command, &mut core).unwrap();

        let select_command = get_select_command_by_kv(
            INDEX_FIELD.to_string().as_bytes().to_vec(),
            ValData::Float64(get_random_age() as f64),
        );
        execute(select_command, &mut core).unwrap();
    }

    fn benchmark_single_insert_with_index() {
        let mut core = ImmuxDBCore::new(&KeyValueEngine::Rocks, NAMESPACE.as_bytes()).unwrap();
        let create_index_command = get_create_index_command(INDEX_FIELD);
        execute(create_index_command, &mut core).unwrap();

        let id_range = (1..100);
        let command = get_batch_json_insert_command(id_range.clone(), true);
        execute(command, &mut core).unwrap();
    }
}
