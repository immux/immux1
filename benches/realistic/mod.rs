mod berka99;
mod census90;
mod utils;

use std::error::Error;

use berka99::berka99;
use census90::census90;
use immuxdb_dev_utils::launch_db;

use std::thread;
use std::time::Duration;

pub struct BenchSpec {
    pub name: &'static str,
    pub unicus_port: u16,
    pub main: &'static dyn Fn(&BenchSpec) -> Result<(), Box<dyn Error>>,
    pub row_limit: usize,
    pub report_period: usize,
}

pub fn bench(benches: Vec<BenchSpec>) {
    for bench in benches {
        println!(
            "\nExecuting bench {}, with tables truncated at row {}",
            bench.name, bench.row_limit
        );

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

fn main() {
    let benches: Vec<BenchSpec> = vec![
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
    bench(benches);
}
