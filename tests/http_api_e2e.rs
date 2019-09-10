#[cfg(test)]
use std::error::Error;
use std::thread;

use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use immuxdb_dev_utils::{launch_db, notified_sleep};
use libimmuxdb::declarations::basics::{ChainName, GroupingLabel, Unit, UnitContent, UnitId};
use libimmuxdb::storage::vkv::ChainHeight;

#[test]
fn e2e_change_database_namespace() -> Result<(), Box<dyn Error>> {
    let db_port = 20001;

    thread::spawn(move || launch_db("e2e_change_database_namespace", db_port));
    notified_sleep(5);

    let client = ImmuxDBClient::new(&format!("localhost:{}", db_port))?;

    let id = UnitId::new(0);
    let grouping = GroupingLabel::new("GROUPING".as_bytes());

    let chain_name_a = ChainName::from("immuxtest-ns-A");
    let data_in_a = UnitContent::String("data-A".to_string());
    let unit_a = Unit {
        id,
        content: data_in_a.clone(),
    };

    let chain_name_b = ChainName::from("immuxtest-ns-B");
    let data_in_b = UnitContent::String("data-B".to_string());
    let unit_b = Unit {
        id,
        content: data_in_b.clone(),
    };

    assert_ne!(chain_name_a, chain_name_b);
    assert_ne!(data_in_a, data_in_b);

    client.switch_chain(&chain_name_a)?;
    client.set_unit(&grouping, &unit_a)?;

    client.switch_chain(&chain_name_b)?;
    client.set_unit(&grouping, &unit_b)?;

    let data_out_b = client.get_by_id(&grouping, &id)?;
    assert_eq!(data_in_b.to_string(), data_out_b.to_string());

    client.switch_chain(&chain_name_a)?;
    let data_out_a = client.get_by_id(&grouping, &id)?;
    assert_eq!(data_in_a.to_string(), data_out_a.to_string());

    Ok(())
}

const INITIAL_HEIGHT: u64 = 1; // The height 0 is empty; hence first data starts at height 1.

#[test]
fn e2e_single_document_versioning() -> Result<(), Box<dyn Error>> {
    let db_port = 20002;

    thread::spawn(move || launch_db("e2e_single_document_versioning", db_port));
    notified_sleep(5);

    let client = ImmuxDBClient::new(&format!("localhost:{}", db_port))?;
    let chain_name = ChainName::from("immuxtest-single-document-versioning");
    client.switch_chain(&chain_name)?;
    let id = UnitId::new(1);
    let grouping = GroupingLabel::new("GROUPING".as_bytes());

    fn dummy_data(height: u64) -> String {
        format!("data-at-height-{}", height)
    }

    let range = INITIAL_HEIGHT..100;

    for height in range.clone() {
        let unit = Unit {
            id,
            content: UnitContent::String(dummy_data(height)),
        };
        client.set_unit(&grouping, &unit)?;
    }

    println!("A");
    let body_text = client.inspect_by_id(&grouping, &id)?;
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
        let chain_height = ChainHeight::new(target_height);
        client.revert_by_id(&grouping, &id, &chain_height)?;
        let current_value = client.get_by_id(&grouping, &id)?;
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
    let chian_name = ChainName::from("immuxtest-multiple-document-versioning");
    client.switch_chain(&chian_name)?;

    let grouping = GroupingLabel::new("GROUPING".as_bytes());

    let inputs: Vec<(Unit)> = vec![
        //id, data
        Unit {
            id: UnitId::new(0),
            content: UnitContent::String("a1".to_string()),
        },
        Unit {
            id: UnitId::new(0),
            content: UnitContent::String("a2".to_string()),
        },
        Unit {
            id: UnitId::new(1),
            content: UnitContent::String("b1".to_string()),
        },
        Unit {
            id: UnitId::new(0),
            content: UnitContent::String("a3".to_string()),
        },
        Unit {
            id: UnitId::new(2),
            content: UnitContent::String("c1".to_string()),
        },
        Unit {
            id: UnitId::new(1),
            content: UnitContent::String("b2".to_string()),
        },
        Unit {
            id: UnitId::new(2),
            content: UnitContent::String("c2".to_string()),
        },
    ];

    // Store data in specified order
    for input in inputs.iter() {
        client.set_unit(&grouping, input)?;
    }

    // Revert in input order and check consistency
    for (index, input) in inputs.iter().enumerate() {
        let height = INITIAL_HEIGHT + (index as u64);
        let chain_height = ChainHeight::new(height);
        client.revert_by_id(&grouping, &input.id, &chain_height)?;
        let current_data = client.get_by_id(&grouping, &input.id)?;
        assert_eq!(current_data, input.content.to_string());
    }

    Ok(())
}
