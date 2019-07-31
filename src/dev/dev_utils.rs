use std::fs::{create_dir_all, remove_dir_all};
use std::time::Duration;
use std::{io, thread};

use libimmuxdb::config::ImmuxDBConfiguration;
use libimmuxdb::run_immuxdb;

pub fn launch_db(project_name: &str, port: u16) -> io::Result<()> {
    let data_root = format!("/tmp/{}/", project_name);
    println!("Initializing database in {}", data_root);
    create_dir_all(&data_root)?;
    remove_dir_all(&data_root)?;
    println!("Existing test data removed");

    let mut config = ImmuxDBConfiguration::default();
    config.data_root = data_root;
    config.unicus_endpoint = format!("127.0.0.1:{}", port);
    match run_immuxdb(&config) {
        Ok(_) => println!("Database started"),
        Err(error) => {
            println!("Cannot start database: {:?}", error);
        }
    }
    return Ok(());
}

pub fn notified_sleep(sec: u16) -> () {
    println!("Waiting {}s...", sec);
    thread::sleep(Duration::from_secs(5));
}
