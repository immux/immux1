mod berka99;
mod census90;
mod utils;

use std::error::Error;
use std::thread;

use immuxdb_dev_utils::{launch_db, notified_sleep};

use berka99::berka99;
use census90::census90;

pub struct BenchSpec {
    pub name: &'static str,
    pub unicus_port: u16,
    pub main: &'static dyn Fn(&BenchSpec) -> Result<(), Box<dyn Error>>,
    pub row_limit: usize,
    pub report_period: usize,
}

fn main() {
    let bench_specs: Vec<BenchSpec> = vec![
        BenchSpec {
            name: "census90",
            unicus_port: 10001,
            main: &census90,
            row_limit: 20_000,
            report_period: 1_000,
        },
        BenchSpec {
            name: "berka99",
            unicus_port: 10002,
            main: &berka99,
            row_limit: 5_000,
            report_period: 1_000,
        },
    ];

    for bench_spec in bench_specs {
        println!(
            "\nExecuting bench {}, with tables truncated at row {}",
            bench_spec.name, bench_spec.row_limit
        );

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
