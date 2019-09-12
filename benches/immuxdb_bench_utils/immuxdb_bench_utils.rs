use std::error::Error;
use std::fs::read;
use std::io;
use std::num::ParseIntError;
use std::time::Instant;

pub use serde::de::{Deserialize, DeserializeOwned};
pub use serde::ser::Serialize;

use libimmuxdb::declarations::basics::{Unit, UnitContent, UnitId};

pub type UnitList = Vec<Unit>;

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

pub mod least_squares {
    fn average(data: &[f64]) -> f64 {
        let sum: f64 = data.iter().sum();
        return sum / (data.len() as f64);
    }

    // for y = kx+b
    // data: [(x1, y1), (x2, y2)...] -> (k, b)
    pub fn solve(data: &[(f64, f64)]) -> (f64, f64) {
        let xs: Vec<f64> = data.iter().map(|pair| pair.0).collect();
        let ys: Vec<f64> = data.iter().map(|pair| pair.1).collect();
        let x_average = average(&xs);
        let y_average = average(&ys);

        let slope: f64 = {
            let numerator: f64 = {
                let mut sum: f64 = 0.0;
                for i in 0..data.len() {
                    sum += (xs[i] - x_average) * (ys[i] - y_average)
                }
                sum
            };
            let denominator: f64 = {
                let mut sum: f64 = 0.0;
                for i in 0..data.len() {
                    sum += (xs[i] - x_average).powi(2)
                }
                sum
            };
            if denominator == 0.0 {
                0.0
            } else {
                numerator / denominator
            }
        };

        let intercept = y_average - slope * x_average;
        return (slope, intercept);
    }
}
