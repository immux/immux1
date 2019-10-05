use std::error::Error;
use std::fs::read;
use std::io;
use std::num::ParseIntError;
use std::str::FromStr;
use std::thread;
use std::time::Instant;

pub use serde::de::{Deserialize, DeserializeOwned};
pub use serde::ser::Serialize;

use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use immuxdb_dev_utils::{launch_db, notified_sleep};
use libimmuxdb::declarations::basics::{ChainName, GroupingLabel, Unit, UnitContent, UnitId};

use crate::declarations::{ArtificialDataBenchSpec, UnitList};

pub fn launch_db_and_start_bench(bench_spec: &ArtificialDataBenchSpec) {
    println!("\nExecuting bench {}", bench_spec.name);

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
                "took {}ms to execute {} {} operations ({}/{} done), average {:.2}ms per item",
                elapsed,
                report_period,
                operation_name,
                count,
                data.len(),
                average_time
            );
            start = Instant::now();
            times.push((count as f64, average_time));
        }
    }
    Ok(times)
}

pub fn csv_to_json_table<J: DeserializeOwned + Serialize>(
    csv_file_path: &str,
    delimiter: u8,
    row_limit: usize,
) -> Result<UnitList, Box<dyn Error>> {
    let reading = read(csv_file_path)?;
    let mut csv_parsing = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(reading.as_slice());
    let data: UnitList = csv_parsing
        .records()
        .map(|result| -> io::Result<Unit> {
            let record = result?;
            let journal: J = record.deserialize(None)?;
            let string = serde_json::to_string(&journal)?;
            let id = UnitId::new(record[0].parse::<u128>().unwrap_or(0));
            let content = UnitContent::JsonString(string);
            let unit = Unit { id, content };
            Ok(unit)
        })
        .map(|datum| -> Unit {
            match datum {
                Err(_) => {
                    let id = UnitId::new(0);
                    let content = UnitContent::String(String::from("json:error"));
                    let unit = Unit { id, content };
                    return unit;
                }
                Ok(datum) => datum,
            }
        })
        .take(row_limit)
        .collect();
    return Ok(data);
}

pub fn read_usize_from_arguments(position: usize) -> Result<usize, ParseIntError> {
    std::env::args()
        .nth(position)
        .unwrap_or(String::new())
        .parse::<usize>()
}

pub fn verify_units_against_db(
    client: &ImmuxDBClient,
    chain_name: &ChainName,
    grouping_label: &GroupingLabel,
    units: &[Unit],
) -> bool {
    for unit in units {
        let data = client
            .get_by_id(chain_name, grouping_label, &unit.id)
            .unwrap();

        if UnitContent::from_str(&data).unwrap() != unit.content {
            return false;
        }
    }
    return true;
}

pub fn verify_journal_against_db(
    client: &ImmuxDBClient,
    chain_name: &ChainName,
    grouping_label: &GroupingLabel,
    units: &[Unit],
) -> bool {
    let id = units[0].id;
    let data = client
        .inspect_by_id(chain_name, grouping_label, &id)
        .unwrap();
    let height_unit_vec: Vec<(&str, Unit)> = data
        .split("\r\n")
        .filter(|datum| *datum != "")
        .map(|height_content_str| {
            let res_str: Vec<&str> = height_content_str.split("|").collect();
            let height = res_str[0];
            let content = res_str[1];
            let unit = Unit {
                id: id,
                content: UnitContent::from_str(content).unwrap(),
            };
            return (height, unit);
        })
        .collect();

    let mut index = 0;
    let mut height = 2;
    for unit in units {
        if *unit != height_unit_vec[index].1 || height.to_string() != height_unit_vec[index].0 {
            return false;
        }
        index += 1;
        height += 1;
    }
    return true;
}
