use std::error::Error;

use crate::{csv_to_jsons_and_id, measure_iteration, BenchSpec};
use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CensusEntry {
    caseid: u64,
    d_age: u64,
    d_ancstry1: u64,
    d_ancstry2: u64,
    i_avail: u64,
    i_citizen: u64,
    i_class: u64,
    d_depart: u64,
    i_disabl1: u64,
    i_disabl2: u64,
    i_english: u64,
    i_feb55: u64,
    i_fertil: u64,
    d_hispanic: u64,
    d_hour89: u64,
    d_hours: u64,
    i_immigr: u64,
    d_income1: u64,
    d_income2: u64,
    d_income3: u64,
    d_income4: u64,
    d_income5: u64,
    d_income6: u64,
    d_income7: u64,
    d_income8: u64,
    d_industry: u64,
    i_korean: u64,
    i_lang1: u64,
    i_looking: u64,
    i_marital: u64,
    i_may75880: u64,
    i_means: u64,
    i_military: u64,
    i_mobility: u64,
    i_mobillim: u64,
    d_occup: u64,
    i_othrserv: u64,
    i_perscare: u64,
    d_pob: u64,
    d_poverty: u64,
    d_pwgt1: u64,
    i_ragechld: u64,
    d_rearning: u64,
    i_relat1: u64,
    i_relat2: u64,
    i_remplpar: u64,
    i_riders: u64,
    i_rlabor: u64,
    i_rownchld: u64,
    d_rpincome: u64,
    i_rpob: u64,
    i_rrelchld: u64,
    i_rspouse: u64,
    i_rvetserv: u64,
    i_school: u64,
    i_sept80: u64,
    i_sex: u64,
    i_subfam1: u64,
    i_subfam2: u64,
    i_tmpabsnt: u64,
    d_travtime: u64,
    i_vietnam: u64,
    d_week89: u64,
    i_work89: u64,
    i_worklwk: u64,
    i_wwii: u64,
    i_yearsch: u64,
    i_yearwrk: u64,
    d_yrsserv: u64,
}

pub fn census90(spec: &BenchSpec) -> Result<(), Box<dyn Error>> {
    let data = csv_to_jsons_and_id::<CensusEntry>(
        "benches/realistic/data-raw/census1990/USCensus1990.data.csv",
        b',',
        spec.row_limit,
    )?;

    let grouping_name = "GROUPING";
    let client = ImmuxDBClient::new(&format!("localhost:{}", spec.unicus_port))?;

    let insert = || -> Result<(), Box<dyn Error>> {
        measure_iteration(
            &data,
            |datum| {
                client
                    .set_key_value(grouping_name, &datum.0, &datum.1)
                    .map_err(|err| err.into())
            },
            "insert",
            spec.report_period,
        )
    };

    let get = || -> Result<(), Box<dyn Error>> {
        measure_iteration(
            &data,
            |datum| {
                client
                    .get_by_key(grouping_name, &datum.0)
                    .map_err(|err| err.into())
            },
            "get",
            spec.report_period,
        )
    };

    insert()?;
    get()?;
    insert()?;
    get()?;

    Ok(())
}
