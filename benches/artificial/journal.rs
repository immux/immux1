use immuxdb_bench_utils::action_executor::execute;
use immuxdb_bench_utils::declarations::{Action, ArtificialDataBenchSpec};
use immuxdb_bench_utils::toolkits::{
    launch_db_and_start_bench, read_usize_from_arguments, verify_journal_against_db,
};
use libimmuxdb::declarations::basics::{PropertyName, UnitContent, UnitId};

fn main() {
    let row_count = read_usize_from_arguments(1).unwrap_or(10_000);
    let report_period = read_usize_from_arguments(2).unwrap_or(10);
    let verify_correctness = read_usize_from_arguments(3).unwrap_or(0) > 0;

    let bench_spec = ArtificialDataBenchSpec {
        name: "journal",
        chain_name: "default_chain",
        unicus_port: 10099,
        main: &execute,
        actions: vec![Action::Insert {
            table_spec: (
                row_count,
                Box::new(|_row_number| -> UnitId { UnitId::new(1) }),
                vec![Box::new(
                    |row_number, _row_count| -> (PropertyName, UnitContent) {
                        let property_name = PropertyName::from(format!("{}", row_number).as_str());
                        let unit_content = UnitContent::String("1234".to_string());
                        return (property_name, unit_content);
                    },
                )],
            ),
            num_jsons_per_command: 1,
        }],
        verify_correctness,
        verification_fn: &verify_journal_against_db,
        report_period,
    };

    launch_db_and_start_bench(&bench_spec);
}
