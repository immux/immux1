use immuxdb_bench_utils::action_executor::execute;
use immuxdb_bench_utils::data_generator::get_string_with_fix_size;
use immuxdb_bench_utils::declarations::{Action, ArtificialDataBenchSpec};
use immuxdb_bench_utils::toolkits::{
    launch_db_and_start_bench, read_usize_from_arguments, verify_units_against_db,
};
use libimmuxdb::declarations::basics::{PropertyName, UnitContent, UnitId};

fn main() {
    let row_count = read_usize_from_arguments(1).unwrap_or(100_000);
    let num_jsons_per_command = read_usize_from_arguments(2).unwrap_or(100);
    let report_period = read_usize_from_arguments(3).unwrap_or(10);
    let verify_correctness = read_usize_from_arguments(4).unwrap_or(0) > 0;

    let bench_spec = ArtificialDataBenchSpec {
        name: "indexed_set",
        unicus_port: 10099,
        main: &execute,
        chain_name: "default_chain",
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
                    row_count,
                    Box::new(|row_number| -> UnitId { UnitId::new(row_number as u128) }),
                    vec![
                        Box::new(|row_number, row_count| -> (PropertyName, UnitContent) {
                            let duplication_rate = 0.8;
                            let duplication_value_size = 10;
                            let property_name_length = 8;
                            let property_name_string =
                                get_string_with_fix_size(property_name_length, 'S');

                            if (row_number as f64) < (row_count as f64) * duplication_rate {
                                let unit_content_str =
                                    get_string_with_fix_size(duplication_value_size, 'C');
                                return (
                                    PropertyName::from(unit_content_str.as_str()),
                                    UnitContent::String(unit_content_str),
                                );
                            } else {
                                let unit_content_str =
                                    get_string_with_fix_size((row_number % 30) as usize, 'C');
                                return (
                                    PropertyName::from(property_name_string.as_str()),
                                    UnitContent::String(unit_content_str),
                                );
                            }
                        }),
                        Box::new(|row_number, row_count| -> (PropertyName, UnitContent) {
                            let duplication_rate = 0.2;
                            let property_name_string =
                                get_string_with_fix_size((row_number % 30) as usize, 'B');

                            if (row_number as f64) < (row_count as f64) * duplication_rate {
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
                        Box::new(|row_number, row_count| -> (PropertyName, UnitContent) {
                            let duplication_rate = 0.1;
                            let property_name_length = 3;
                            let property_name_string =
                                get_string_with_fix_size(property_name_length, 'F');

                            if (row_number as f64) < (row_count as f64) * duplication_rate {
                                return (
                                    PropertyName::from(property_name_string.as_str()),
                                    UnitContent::Float64(1.0),
                                );
                            } else {
                                let float_number = 100.0 * (row_number as f64).sin().powf(2.0);
                                return (
                                    PropertyName::from(property_name_string.as_str()),
                                    UnitContent::Float64(float_number),
                                );
                            }
                        }),
                    ],
                ),
                num_jsons_per_command,
            },
        ],
        verify_correctness,
        verification_fn: &verify_units_against_db,
        report_period,
    };

    launch_db_and_start_bench(&bench_spec);
}
