mod berka99;
mod census90;

use std::error::Error;
use std::fs::{create_dir_all, read, remove_dir_all};
use std::time::{Duration, Instant};
use std::{io, thread};

use csv;
use immuxdb_dev_utils::{launch_db, notified_sleep};
use serde::de::DeserializeOwned;
use serde::Serialize;

use berka99::berka99;
use census90::census90;

pub struct BenchSpec {
    name: &'static str,
    unicus_port: u16,
    main: &'static dyn Fn(&BenchSpec) -> Result<(), Box<dyn Error>>,
    row_limit: usize,
    report_period: usize,
}

pub type JsonTableWithId = Vec<(String, String)>;

pub fn csv_to_jsons_and_id<J: DeserializeOwned + Serialize>(
    csv_file_path: &str,
    delimiter: u8,
    row_limit: usize,
) -> Result<JsonTableWithId, Box<dyn Error>> {
    let reading = read(csv_file_path)?;
    type ID = String;
    type JsonString = String;
    let mut csv_parsing = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(reading.as_slice());
    let data: Vec<(ID, JsonString)> = csv_parsing
        .records()
        .map(|result| -> io::Result<(String, String)> {
            let record = result?;
            let entry: J = record.deserialize(None)?;
            let string = serde_json::to_string(&entry)?;
            Ok((record[0].to_string(), string))
        })
        .map(|datum| match datum {
            Err(_) => (String::from("id:error"), String::from("json:error")),
            Ok(datum) => datum,
        })
        .take(row_limit)
        .collect();
    return Ok(data);
}

pub fn measure_iteration<D, F>(
    data: &[D],
    operate: F,
    operation_name: &str,
    report_period: usize,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>>
where
    F: Fn(&D) -> Result<String, Box<dyn Error>>,
{
    let mut start = Instant::now();
    let mut count = 0;
    let total_periods = data.len() / report_period;
    let mut times: Vec<(f64, f64)> = Vec::with_capacity(total_periods + 1);
    for datum in data.iter() {
        operate(datum)?;
        count += 1;
        if count % report_period == 0 {
            let elapsed = start.elapsed().as_millis();
            let average_time = elapsed as f64 / report_period as f64;
            println!(
                "took {}ms to execute {} {} operations (out of total {}), average {:.2}ms per item",
                elapsed,
                report_period,
                operation_name,
                data.len(),
                average_time
            );
            start = Instant::now();
            times.push((count as f64, average_time));
        }
    }
    Ok(times)
}

fn main() {
    let benches: Vec<BenchSpec> = vec![
        BenchSpec {
            name: "census90",
            unicus_port: 10001,
            main: &census90,
            row_limit: 1000,
            report_period: 100,
        },
        BenchSpec {
            name: "berka99",
            unicus_port: 10002,
            main: &berka99,
            row_limit: 300,
            report_period: 100,
        },
    ];

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
