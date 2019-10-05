use std::error::Error;

use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use libimmuxdb::declarations::basics::{ChainName, GroupingLabel, PropertyName, Unit, UnitId};

use crate::data_generator::generate_json_table;
use crate::declarations::{Action, ArtificialDataBenchSpec};
use crate::toolkits::measure_iteration;

pub fn execute(bench_spec: &ArtificialDataBenchSpec) -> Result<(), Box<dyn Error>> {
    let client = ImmuxDBClient::new(&format!("localhost:{}", bench_spec.unicus_port))?;

    for action in &bench_spec.actions {
        match action {
            Action::CreateIndex { property_name_vec } => {
                create_index(
                    &client,
                    &property_name_vec,
                    &bench_spec.chain_name,
                    &bench_spec.name,
                    bench_spec.report_period,
                )?;
            }
            Action::Insert {
                table_spec,
                num_jsons_per_command,
            } => {
                let (row_count, unit_id_spec, json_spec) = table_spec;
                let table = generate_json_table(*row_count, &unit_id_spec, &json_spec);
                let units_vec: Vec<Vec<Unit>> = table
                    .chunks(*num_jsons_per_command)
                    .map(|units| units.to_owned())
                    .collect();
                batch_insert(
                    &client,
                    &bench_spec.chain_name,
                    &bench_spec.name,
                    &units_vec,
                    bench_spec.verify_correctness,
                    &bench_spec.verification_fn,
                    bench_spec.report_period,
                )?;
            }
            Action::Select { start, end } => {
                batch_select(
                    &client,
                    &start,
                    &end,
                    &bench_spec.chain_name,
                    &bench_spec.name,
                    bench_spec.report_period,
                )?;
            }
        }
    }
    return Ok(());
}

fn create_index(
    client: &ImmuxDBClient,
    property_name_vec: &[PropertyName],
    chain_name: &str,
    bench_name: &str,
    report_period: usize,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    measure_iteration(
        property_name_vec,
        |property_name| {
            client
                .create_index(
                    &ChainName::from(chain_name),
                    &GroupingLabel::new(bench_name.as_bytes()),
                    property_name,
                )
                .map_err(|err| err.into())
        },
        "create_index",
        report_period,
    )
}

fn batch_select(
    client: &ImmuxDBClient,
    start: &UnitId,
    end: &UnitId,
    chain_name: &str,
    bench_name: &str,
    report_period: usize,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    let ids: Vec<UnitId> = (start.as_int()..end.as_int())
        .map(|id| UnitId::new(id))
        .collect();

    measure_iteration(
        &ids,
        |id| {
            client
                .get_by_id(
                    &ChainName::from(chain_name),
                    &GroupingLabel::new(bench_name.as_bytes()),
                    id,
                )
                .map_err(|err| err.into())
        },
        "batch_select",
        report_period,
    )
}

fn batch_insert<F>(
    client: &ImmuxDBClient,
    chain_name: &str,
    bench_name: &str,
    units_vec: &[Vec<Unit>],
    verify_correctness: bool,
    verification_fn: F,
    report_period: usize,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>>
where
    F: Fn(&ImmuxDBClient, &ChainName, &GroupingLabel, &[Unit]) -> bool,
{
    let res = measure_iteration(
        &units_vec,
        |units| {
            client
                .set_batch_units(
                    &ChainName::from(chain_name),
                    &GroupingLabel::new(bench_name.as_bytes()),
                    units,
                )
                .map_err(|err| err.into())
        },
        "batch_insert",
        report_period,
    )?;

    if verify_correctness {
        let units: Vec<Unit> = units_vec
            .iter()
            .flat_map(|units| units.to_owned())
            .collect();
        let verification_result = verification_fn(
            client,
            &ChainName::from(chain_name),
            &GroupingLabel::new(bench_name.as_bytes()),
            &units,
        );
        assert_eq!(verification_result, true);

        println!("Database entries match input tables");
    } else {
        println!("Data verification is skipped");
    }

    return Ok(res);
}
