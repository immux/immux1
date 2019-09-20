use std::error::Error;

use immuxdb_client::ImmuxDBClient;
use libimmuxdb::declarations::basics::{GroupingLabel, NameProperty, PropertyName, Unit, UnitId};

pub type UnitList = Vec<Unit>;

pub type PropertySpec = Box<dyn Fn(u64, usize) -> NameProperty>;
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
    pub verify_correctness: bool,
    pub verification_fn: &'static dyn Fn(&ImmuxDBClient, &GroupingLabel, &[Unit]) -> bool,
    pub report_period: usize,
}
