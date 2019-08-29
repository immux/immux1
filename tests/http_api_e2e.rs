#[cfg(test)]
use std::error::Error;
use std::thread;

use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use immuxdb_dev_utils::{launch_db, notified_sleep};

#[test]
fn e2e_change_database_namespace() -> Result<(), Box<dyn Error>> {
    let db_port = 20001;

    thread::spawn(move || launch_db("e2e_change_database_namespace", db_port));
    notified_sleep(5);

    let client = ImmuxDBClient::new(&format!("localhost:{}", db_port))?;

    let id = 0;
    let grouping = "GROUPING";

    let namespace_a = "immuxtest-ns-A";
    let data_in_a = "data-A";

    let namespace_b = "immuxtest-ns-B";
    let data_in_b = "data-B";

    assert_ne!(namespace_a, namespace_b);
    assert_ne!(data_in_a, data_in_b);

    client.switch_namespace(namespace_a)?;
    client.set_by_id(grouping, id, data_in_a)?;

    client.switch_namespace(namespace_b)?;
    client.set_by_id(grouping, id, data_in_b)?;

    let data_out_b = client.get_by_id(grouping, id)?;
    assert_eq!(data_in_b, data_out_b);

    client.switch_namespace(namespace_a)?;
    let data_out_a = client.get_by_id(grouping, id)?;
    assert_eq!(data_in_a, data_out_a);

    Ok(())
}

const INITIAL_HEIGHT: u64 = 1; // The height 0 is empty; hence first data starts at height 1.

#[test]
fn e2e_single_document_versioning() -> Result<(), Box<dyn Error>> {
    let db_port = 20002;

    thread::spawn(move || launch_db("e2e_single_document_versioning", db_port));
    notified_sleep(5);

    let client = ImmuxDBClient::new(&format!("localhost:{}", db_port))?;
    client.switch_namespace("immuxtest-single-document-versioning")?;
    let id = 1;
    let grouping = "GROUPING";

    fn dummy_data(height: u64) -> String {
        format!("data-at-height-{}", height)
    }

    let range = INITIAL_HEIGHT..100;

    for height in range.clone() {
        client.set_by_id(grouping, id, &dummy_data(height))?;
    }

    println!("A");
    let body_text = client.inspect_by_id(grouping, id)?;
    println!("Output text {}", body_text);
    let data: Vec<(&str, &str, &str)> = body_text
        .split("\r\n")
        .filter(|line| line.len() > 0)
        .map(|line| {
            let segments: Vec<_> = line.split("|").collect();
            return (segments[0], segments[1], segments[2]);
        })
        .collect();

    println!("B");
    // Test inspection of version changes
    for expected_height in range.clone() {
        let index = (expected_height - INITIAL_HEIGHT) as usize;
        let (_actual_deleted, actual_height, actual_data) = data[index];
        let expected_data = dummy_data(expected_height);
        assert_eq!(expected_height, actual_height.parse::<u64>().unwrap());
        assert_eq!(expected_data, actual_data);
    }

    println!("C");
    // Test revert
    for target_height in range.clone() {
        client.revert_by_id(grouping, id, target_height)?;
        let current_value = client.get_by_id(grouping, id)?;
        let expected_value = dummy_data(target_height);
        assert_eq!(current_value, expected_value);
    }

    Ok(())
}

#[test]
fn e2e_multiple_document_versioning() -> Result<(), Box<dyn Error>> {
    let db_port = 20003;

    thread::spawn(move || launch_db("e2e_multiple_document_versioning", db_port));
    notified_sleep(5);

    let client = ImmuxDBClient::new(&format!("localhost:{}", db_port))?;
    client.switch_namespace("immuxtest-multiple-document-versioning")?;

    let grouping = "GROUPING";

    let inputs: Vec<(u128, &str)> = vec![
        //id, data
        (0, "a1"),
        (0, "a2"),
        (1, "b1"),
        (0, "a3"),
        (2, "c1"),
        (1, "b2"),
        (2, "c2"),
    ];

    // Store data in specified order
    for input in inputs.iter() {
        let (id, data) = input;
        client.set_by_id(grouping, *id, data)?;
    }

    // Revert in input order and check consistency
    for (index, input) in inputs.iter().enumerate() {
        let height = INITIAL_HEIGHT + (index as u64);
        let (id, input_data) = input;
        client.revert_by_id(grouping, *id, height)?;
        let current_data = client.get_by_id(grouping, *id)?;
        assert_eq!(&current_data, input_data);
    }

    Ok(())
}
