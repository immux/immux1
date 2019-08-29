use std::error::Error;
use std::thread;

use immuxdb_dev_utils::{launch_db, notified_sleep};

pub struct BenchSpec {
    pub name: &'static str,
    pub unicus_port: u16,
    pub main: &'static dyn Fn(&BenchSpec) -> Result<(), Box<dyn Error>>,
    pub row_limit: usize,
    pub report_period: usize,
}

pub fn bench_all(benches: &[BenchSpec]) {
    for bench in benches {
        println!(
            "\nExecuting bench {}, with tables truncated at row {}",
            bench.name, bench.row_limit
        );

        let bench_name = bench.name;
        let db_port = bench.unicus_port;
        thread::spawn(move || launch_db(bench_name, db_port));
        notified_sleep(5);

        println!("Start benching...");
        let f = bench.main;
        match f(&bench) {
            Err(error) => eprintln!("Failed to bench {}: {:?}", bench.name, error),
            Ok(_) => {}
        }
    }
}
