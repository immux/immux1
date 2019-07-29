use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{csv_to_jsons_and_id, measure_iteration, BenchSpec, JsonTableWithId};
use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};

#[derive(Debug, Deserialize, Serialize)]
struct Account {
    account_id: u16,
    district_id: u8,
    frequency: String,
    date: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Card {
    card_id: u16,
    disp_id: u16,
    r#type: String,
    issued: String, // date
}

#[derive(Debug, Deserialize, Serialize)]
struct Client {
    client_id: u16,
    birth_number: String,
    district_id: u8,
}

#[derive(Debug, Deserialize, Serialize)]
struct Disp {
    disp_id: u16,
    client_id: u16,
    account_id: u16,
    r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct District {
    code: u8,
    name: String,
    region: String,
    inhabitant_number: u32,
    municipalities_inhabitants_0_499: u32,
    municipalities_inhabitants_500_1999: u32,
    municipalities_inhabitants_2000_9999: u32,
    municipalities_inhabitants_10000_inifnity: u32,
    city_numbre: u16,
    ratio_urban_inhabitants: f64,
    average_salary: u32,
    unimployment_rate_95: f64,
    unimployment_rate_96: f64,
    enterpreneurs_per_1000: u16,
    crime_number_95: u32,
    crime_number_96: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Loan {
    loan_id: u16,
    account_id: u16,
    date: String,
    amount: u32,
    duration: u16,
    payments: f64,
    status: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Order {
    order_id: u16,
    account_id: u16,
    bank_to: String,
    account_to: String,
    amount: f64,
    k_symbol: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Trans {
    trans_id: u32,
    account_id: u16,
    date: String,
    r#type: String,
    operation: String,
    amount: f64,
    balance: f64,
    k_symbol: String,
    bank: String,
    account: String,
}

pub fn berka99(bench: &BenchSpec) -> Result<(), Box<dyn Error>> {
    let paths = vec![
        "account", "card", "client", "disp", "district", "loan", "order", "trans",
    ];
    let dataset: Vec<(String, JsonTableWithId)> = paths
        .iter()
        .map(
            |table_name| -> (String, Result<JsonTableWithId, Box<dyn Error>>) {
                let csv_path = format!("benches/realistic/data-raw/berka99/{}.asc", table_name);
                let data = match table_name.as_ref() {
                    "account" => csv_to_jsons_and_id::<Account>(&csv_path, b';', bench.row_limit),
                    "card" => csv_to_jsons_and_id::<Card>(&csv_path, b';', bench.row_limit),
                    "client" => csv_to_jsons_and_id::<Client>(&csv_path, b';', bench.row_limit),
                    "disp" => csv_to_jsons_and_id::<Disp>(&csv_path, b';', bench.row_limit),
                    "district" => csv_to_jsons_and_id::<District>(&csv_path, b';', bench.row_limit),
                    "loan" => csv_to_jsons_and_id::<Loan>(&csv_path, b';', bench.row_limit),
                    "order" => csv_to_jsons_and_id::<Order>(&csv_path, b';', bench.row_limit),
                    "trans" => csv_to_jsons_and_id::<Trans>(&csv_path, b';', bench.row_limit),
                    _ => panic!("Unexpected table {}", table_name),
                };
                return (table_name.to_string(), data);
            },
        )
        .map(|result| -> (String, JsonTableWithId) {
            match result.1 {
                Err(error) => {
                    eprintln!("CSV error: {}", error);
                    return (String::from("error"), vec![]);
                }
                Ok(table) => return (result.0, table),
            }
        })
        .collect();

    let client = ImmuxDBClient::new(&format!("localhost:{}", bench.unicus_port))?;

    for table in dataset {
        println!("Loading table '{}'", table.0);
        measure_iteration(
            &table.1,
            |row| {
                client
                    .set_key_value(bench.name, &row.0, &row.1)
                    .map_err(|err| err.into())
            },
            "get",
            bench.report_period,
        )?;
    }

    return Ok(());
}
