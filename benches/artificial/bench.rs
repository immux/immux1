use std::error::Error;

use immuxdb_bench_utils::measure_iteration;
use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use libimmuxdb::declarations::basics::{GroupingLabel, NameProperty, PropertyName, Unit, UnitId};

use crate::data_generator::generate_json_table;

pub type PropertySpec = Box<dyn Fn(&UnitId, usize) -> NameProperty>;
pub type JsonSpec = Vec<PropertySpec>;
pub type UnitIdSpec = Box<dyn Fn(u64) -> UnitId>;
pub type RowCount = usize;
pub type JsonTableSpec = (RowCount, UnitIdSpec, JsonSpec);

pub enum Action {
    CreateIndex {
        property_name_vec: Vec<PropertyName>,
    },
    Insert {
        table_spec: JsonTableSpec,
        num_jsons_per_command: usize,
    },
    Select {
        start: UnitId,
        end: UnitId,
    },
}

pub struct ArtificialDataBenchSpec {
    pub name: &'static str,
    pub unicus_port: u16,
    pub main: &'static dyn Fn(&ArtificialDataBenchSpec) -> Result<(), Box<dyn Error>>,
    pub actions: Vec<Action>,
    pub report_period: usize,
}

pub fn execute(bench_spec: &ArtificialDataBenchSpec) -> Result<(), Box<dyn Error>> {
    let client = ImmuxDBClient::new(&format!("localhost:{}", bench_spec.unicus_port))?;

    for action in &bench_spec.actions {
        match action {
            Action::CreateIndex { property_name_vec } => {
                create_index(
                    &client,
                    &property_name_vec,
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
                    &units_vec,
                    &bench_spec.name,
                    bench_spec.report_period,
                )?;
            }
            Action::Select { start, end } => {
                batch_select(
                    &client,
                    &start,
                    &end,
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
    bench_name: &str,
    report_period: usize,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    measure_iteration(
        property_name_vec,
        |property_name| {
            client
                .create_index(&GroupingLabel::new(bench_name.as_bytes()), property_name)
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
                .get_by_id(&GroupingLabel::new(bench_name.as_bytes()), id)
                .map_err(|err| err.into())
        },
        "batch_select",
        report_period,
    )
}

fn batch_insert(
    client: &ImmuxDBClient,
    units_vec: &[Vec<Unit>],
    bench_name: &str,
    report_period: usize,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    measure_iteration(
        &units_vec,
        |units| {
            client
                .set_batch_units(&GroupingLabel::new(bench_name.as_bytes()), units)
                .map_err(|err| err.into())
        },
        "batch_insert",
        report_period,
    )
}
