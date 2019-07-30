use std::env;

use libimmuxdb::config::ImmuxDBConfiguration;
use libimmuxdb::run_immuxdb;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = ImmuxDBConfiguration::compile_from_args(&args);
    match run_immuxdb(&config) {
        Err(error) => eprintln!("ImmuxDB failed: {:#?}", error),
        Ok(_) => (),
    }
}
