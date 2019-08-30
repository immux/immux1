mod data_generator;

use data_generator::{generate_json_table, get_string_with_fix_size};
use immuxdb_bench_utils::{measure_iteration, JsonTable};
use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use immuxdb_dev_utils::launch_db;
use libimmuxdb::declarations::basics::{
    GroupingLabel, NameProperty, PropertyName, Unit, UnitContent, UnitId,
};

use std::error::Error;
use std::thread;
use std::time::Duration;

pub enum Action {
    GenerateTable {
        row_count: usize,
        unit_id_spec: UnitIDSpec,
        json_spec: Vec<PropertySpec>,
    },
    CreateIndex,
    Insert {
        num_jsons_per_command: usize,
    },
    Select {
        start: UnitId,
        end: UnitId,
    },
}

pub type PropertySpec = Box<dyn Fn(&UnitId, usize) -> NameProperty>;
pub type UnitIDSpec = Box<dyn Fn(u64) -> UnitId>;

pub struct ArtificialDataBenchSpec {
    pub name: &'static str,
    pub unicus_port: u16,
    pub main: &'static dyn Fn(&ArtificialDataBenchSpec) -> Result<(), Box<dyn Error>>,
    pub actions: Vec<Action>,
    pub report_period: usize,
}

fn main() {
    let benche_specs = vec![ArtificialDataBenchSpec {
        name: "artifical_data_benchmark",
        unicus_port: 10099,
        main: &execute_actions,
        actions: vec![
            Action::GenerateTable {
                row_count: 5000,
                unit_id_spec: Box::new(|_row_number| -> UnitId { UnitId::new(1) }),
                json_spec: vec![
                    Box::new(|id, row_count| -> (PropertyName, UnitContent) {
                        let id_int = id.as_int();
                        let duplication_rate = 0.8;
                        let duplication_value_size = 10;
                        let property_name_length = 8;
                        let property_name_string =
                            get_string_with_fix_size(property_name_length, 'S');

                        if (id_int as f64) < (row_count as f64) * duplication_rate {
                            let unit_content_str =
                                get_string_with_fix_size(duplication_value_size, 'C');
                            return (
                                PropertyName::from(unit_content_str.as_str()),
                                UnitContent::String(unit_content_str),
                            );
                        } else {
                            let unit_content_str =
                                get_string_with_fix_size((id_int % 30) as usize, 'C');
                            return (
                                PropertyName::from(property_name_string.as_str()),
                                UnitContent::String(unit_content_str),
                            );
                        }
                    }),
                    Box::new(|id, row_count| -> (PropertyName, UnitContent) {
                        let id_int = id.as_int();
                        let duplication_rate = 0.2;
                        let property_name_string =
                            get_string_with_fix_size((id_int % 30) as usize, 'B');

                        if (id_int as f64) < (row_count as f64) * duplication_rate {
                            return (
                                PropertyName::from(property_name_string.as_str()),
                                UnitContent::Bool(true),
                            );
                        } else {
                            return (
                                PropertyName::from(property_name_string.as_str()),
                                UnitContent::Bool(false),
                            );
                        }
                    }),
                    Box::new(|id, row_count| -> (PropertyName, UnitContent) {
                        let id_int = id.as_int();
                        let duplication_rate = 0.1;
                        let property_name_length = 3;
                        let property_name_string =
                            get_string_with_fix_size(property_name_length, 'F');

                        if (id_int as f64) < (row_count as f64) * duplication_rate {
                            return (
                                PropertyName::from(property_name_string.as_str()),
                                UnitContent::Float64(1.0),
                            );
                        } else {
                            let float_number = 100.0 * (id_int as f64).sin().powf(2.0);
                            return (
                                PropertyName::from(property_name_string.as_str()),
                                UnitContent::Float64(float_number),
                            );
                        }
                    }),
                ],
            },
            Action::CreateIndex,
            Action::Insert {
                num_jsons_per_command: 100,
            },
        ],
        report_period: 10,
    }];

    bench(benche_specs);
}

pub fn execute_actions(bench: &ArtificialDataBenchSpec) -> Result<(), Box<dyn Error>> {
    let client = ImmuxDBClient::new(&format!("localhost:{}", bench.unicus_port))?;
    let mut table: JsonTable = vec![];
    let mut property_name_vec: Vec<PropertyName> = vec![];

    for action in bench.actions.iter() {
        match action {
            Action::GenerateTable {
                row_count,
                unit_id_spec,
                json_spec,
            } => {
                let (new_table, new_property_name_vec) =
                    generate_json_table(row_count, unit_id_spec, json_spec);
                table = new_table;
                property_name_vec = new_property_name_vec;
            }
            Action::CreateIndex => {
                create_index(&property_name_vec, &client, bench)?;
            }
            Action::Insert {
                num_jsons_per_command,
            } => {
                batch_insert(bench, &client, &table, num_jsons_per_command)?;
            }
            Action::Select { start, end } => {
                batch_select(bench, &client, start, end)?;
            }
        }
    }
    return Ok(());
}

fn create_index(
    property_name_vec: &Vec<PropertyName>,
    client: &ImmuxDBClient,
    bench: &ArtificialDataBenchSpec,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    measure_iteration(
        property_name_vec,
        |property_name| {
            client
                .create_index(&GroupingLabel::new(bench.name.as_bytes()), property_name)
                .map_err(|err| err.into())
        },
        "create_index",
        bench.report_period,
    )
}

fn batch_select(
    bench: &ArtificialDataBenchSpec,
    client: &ImmuxDBClient,
    start: &UnitId,
    end: &UnitId,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    let mut ids = vec![];
    for i in start.as_int()..end.as_int() {
        ids.push(UnitId::new(i));
    }

    measure_iteration(
        &ids,
        |id| {
            client
                .get_by_id(&GroupingLabel::new(bench.name.as_bytes()), id)
                .map_err(|err| err.into())
        },
        "batch_select",
        bench.report_period,
    )
}

fn batch_insert(
    bench: &ArtificialDataBenchSpec,
    client: &ImmuxDBClient,
    table: &[Unit],
    num_jsons_per_command: &usize,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    let units_vec: Vec<Vec<Unit>> = table
        .chunks(*num_jsons_per_command)
        .map(|units| units.to_owned())
        .collect();

    measure_iteration(
        &units_vec,
        |units| {
            client
                .set_batch_units(&GroupingLabel::new(bench.name.as_bytes()), units)
                .map_err(|err| err.into())
        },
        "batch_insert",
        bench.report_period,
    )
}

pub fn bench(benches: Vec<ArtificialDataBenchSpec>) {
    for bench in benches {
        println!("\nExecuting bench {}", bench.name);

        let bench_name = bench.name;
        let db_port = bench.unicus_port;
        thread::spawn(move || launch_db(bench_name, db_port));
        println!("Waiting 5s for database to be ready...");
        thread::sleep(Duration::from_secs(5));

        println!("Start benching...");
        let f = bench.main;
        match f(&bench) {
            Err(error) => eprintln!("Failed to bench {}: {:?}", bench.name, error),
            Ok(_) => {}
        }
    }
}
