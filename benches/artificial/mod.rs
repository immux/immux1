mod bench;
mod data_generator;

use std::thread;

use immuxdb_dev_utils::{launch_db, notified_sleep};
use libimmuxdb::declarations::basics::{PropertyName, UnitContent, UnitId};

use crate::bench::{execute, Action, ArtificialDataBenchSpec};
use crate::data_generator::get_string_with_fix_size;

fn main() {
    let bench_specs = vec![ArtificialDataBenchSpec {
        name: "artificial_data_benchmark",
        unicus_port: 10099,
        main: &execute,
        actions: vec![
            Action::CreateIndex {
                property_name_vec: vec![
                    PropertyName::from("B"),
                    PropertyName::from("BB"),
                    PropertyName::from("BBB"),
                    PropertyName::from("BBBB"),
                    PropertyName::from("BBBBB"),
                    PropertyName::from("BBBBBB"),
                    PropertyName::from("BBBBBBB"),
                    PropertyName::from("BBBBBBBB"),
                    PropertyName::from("BBBBBBBBB"),
                    PropertyName::from("BBBBBBBBBB"),
                    PropertyName::from("BBBBBBBBBBB"),
                    PropertyName::from("BBBBBBBBBBBB"),
                    PropertyName::from("BBBBBBBBBBBBB"),
                    PropertyName::from("FFF"),
                    PropertyName::from("CCCCCCCCCC"),
                ],
            },
            Action::Insert {
                table_spec: (
                    5000,
                    Box::new(|_row_number| -> UnitId { UnitId::new(1) }),
                    vec![
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
                ),
                num_jsons_per_command: 100,
            },
        ],
        report_period: 10,
    }];

    for bench_spec in bench_specs {
        println!("\nExecuting bench {}", bench_spec.name);

        let bench_name = bench_spec.name;
        let db_port = bench_spec.unicus_port;
        thread::spawn(move || launch_db(bench_name, db_port));
        notified_sleep(5);

        println!("Start benching...");
        let f = bench_spec.main;
        match f(&bench_spec) {
            Err(error) => eprintln!("Failed to bench {}: {:?}", bench_spec.name, error),
            Ok(_) => {}
        }
    }
}
