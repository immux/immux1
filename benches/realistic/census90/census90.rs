use std::error::Error;
use std::thread;

use serde::{Deserialize, Serialize};

use immuxdb_bench_utils::least_squares::solve;
use immuxdb_bench_utils::toolkits::{
    csv_to_json_table, measure_iteration, read_usize_from_arguments,
};
use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use immuxdb_dev_utils::{launch_db, notified_sleep};
use libimmuxdb::declarations::basics::GroupingLabel;

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

fn main() {
    let bench_name = "census90";
    let row_limit = read_usize_from_arguments(1).unwrap_or(20_000);
    let report_period = read_usize_from_arguments(2).unwrap_or(1_000);
    let verify_correctness = read_usize_from_arguments(3).unwrap_or(0) > 0;
    let db_port = 18000;

    println!(
        "\nExecuting bench {}, with tables truncated at row {}, aggregating {} operations",
        bench_name, row_limit, report_period
    );
    if verify_correctness {
        println!("Verification of 'get' results are turned on")
    } else {
        println!("Database outputs are ignored and not verified")
    }

    thread::spawn(move || launch_db(bench_name, db_port));
    notified_sleep(5);

    let table = csv_to_json_table::<CensusEntry>(
        "benches/realistic/census90/data-raw/USCensus1990.data.txt",
        b',',
        row_limit,
    )
    .unwrap();

    let grouping_label = GroupingLabel::new("GROUPING".as_bytes());
    let client = ImmuxDBClient::new(&format!("localhost:{}", db_port)).unwrap();

    let insert = || -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
        measure_iteration(
            &table,
            |unit| {
                client
                    .set_unit(&grouping_label, unit)
                    .map_err(|err| err.into())
            },
            "insert",
            report_period,
        )
    };

    let get = || -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
        measure_iteration(
            &table,
            |unit| {
                client
                    .get_by_id(&grouping_label, &unit.id)
                    .map(|output| {
                        if verify_correctness {
                            let original = &unit.content;
                            assert_eq!(original.to_string(), output);
                        };
                        output
                    })
                    .map_err(|err| err.into())
            },
            "get",
            report_period,
        )
    };

    let time_insert_1 = insert().unwrap();
    let (k_insert_1, _) = solve(&time_insert_1);
    println!("Gained {:.2e} ms per item on average", k_insert_1);

    let time_get_1 = get().unwrap();
    let (k_get_1, _) = solve(&time_get_1);
    println!("Gained {:.2e} ms per item on average", k_get_1);
}
