use std::error::Error;
use std::thread;

use serde::{Deserialize, Serialize};

use immuxdb_bench_utils::declarations::UnitList;
use immuxdb_bench_utils::toolkits::{
    csv_to_json_table, measure_iteration, read_usize_from_arguments, verify_units_against_db,
};
use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use immuxdb_dev_utils::{launch_db, notified_sleep};
use libimmuxdb::declarations::basics::{ChainName, GroupingLabel};

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

fn main() {
    let bench_name = "berka99";
    let row_limit = read_usize_from_arguments(1).unwrap_or(20_000);
    let report_period = read_usize_from_arguments(2).unwrap_or(1_000);
    let verify_correctness = read_usize_from_arguments(3).unwrap_or(0) > 0;
    let port = 18001;

    println!(
        "\nExecuting bench {}, with tables truncated at row {}, aggregating {} operations",
        bench_name, row_limit, report_period
    );

    thread::spawn(move || launch_db("berka99", port));
    notified_sleep(5);

    let paths = vec![
        "account", "card", "client", "disp", "district", "loan", "order", "trans",
    ];
    let dataset: Vec<(String, UnitList)> = paths
        .iter()
        .map(|table_name| -> (String, Result<UnitList, Box<dyn Error>>) {
            let csv_path = format!("benches/realistic/berka99/data-raw/{}.asc", table_name);
            let data = match table_name.as_ref() {
                "account" => csv_to_json_table::<Account>(&csv_path, b';', row_limit),
                "card" => csv_to_json_table::<Card>(&csv_path, b';', row_limit),
                "client" => csv_to_json_table::<Client>(&csv_path, b';', row_limit),
                "disp" => csv_to_json_table::<Disp>(&csv_path, b';', row_limit),
                "district" => csv_to_json_table::<District>(&csv_path, b';', row_limit),
                "loan" => csv_to_json_table::<Loan>(&csv_path, b';', row_limit),
                "order" => csv_to_json_table::<Order>(&csv_path, b';', row_limit),
                "trans" => csv_to_json_table::<Trans>(&csv_path, b';', row_limit),
                _ => panic!("Unexpected table {}", table_name),
            };
            return (table_name.to_string(), data);
        })
        .map(|result| -> (String, UnitList) {
            match result.1 {
                Err(error) => {
                    eprintln!("CSV error: {}", error);
                    return (String::from("error"), vec![]);
                }
                Ok(table) => return (result.0, table),
            }
        })
        .collect();

    let client = ImmuxDBClient::new(&format!("localhost:{}", port)).unwrap();
    let chain_name = ChainName::from("realistic_bench");
    for (table_name, table) in dataset.iter() {
        println!("Loading table '{}'", table_name);
        let grouping_label = GroupingLabel::from(table_name.as_str());
        measure_iteration(
            table,
            |unit| {
                client
                    .set_unit(&chain_name, &grouping_label, &unit)
                    .map_err(|err| err.into())
            },
            "get",
            report_period,
        )
        .unwrap();
    }

    if verify_correctness {
        for (table_name, table) in dataset.iter() {
            println!("Verifying table '{}'", table_name);
            let grouping_label = GroupingLabel::from(table_name.as_str());
            let verification_result =
                verify_units_against_db(&client, &chain_name, &grouping_label, table);
            assert_eq!(verification_result, true);
        }
        println!("Database entries match input tables")
    } else {
        println!("Data verification is skipped")
    }
}
