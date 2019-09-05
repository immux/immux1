use std::error::Error;
use std::fs::read;
use std::io;

use csv;
use immuxdb_bench_utils::UnitList;
use libimmuxdb::declarations::basics::{Unit, UnitContent, UnitId};
use serde::de::DeserializeOwned;
use serde::Serialize;

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
