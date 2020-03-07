#[cfg(test)]
use std::error::Error;
use std::thread;

use serde_json::Value as JsonValue;

use immuxdb_client::{ImmuxDBClient, ImmuxDBConnector};
use immuxdb_dev_utils::{launch_db, notified_sleep};
use libimmuxdb::declarations::basics::{ChainName, GroupingLabel, Unit, UnitContent, UnitId};
use libimmuxdb::declarations::basics::{NameProperty, PropertyName};
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

    let body_text = client.inspect_by_id(&grouping, &id)?;
    let data: Vec<(&str, &str)> = body_text
        .split("\r\n")
        .filter(|line| line.len() > 0)
        .map(|line| {
            let segments: Vec<_> = line.split("|").collect();
            return (segments[0], segments[1]);
        })
        .collect();

    // Test inspection of version changes
    for expected_height in range.clone() {
        let index = (expected_height - INITIAL_HEIGHT) as usize;
        let (actual_height, actual_data) = data[index];
        let expected_data = dummy_data(expected_height);
        assert_eq!(expected_height, actual_height.parse::<u64>().unwrap());
        assert_eq!(expected_data, actual_data);
    }

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
    let chain_name = ChainName::from("immuxtest-multiple-document-versioning");
    client.switch_chain(&chain_name)?;

    let grouping = GroupingLabel::new("GROUPING".as_bytes());

    let units: Vec<Unit> = [
        //id, data
        (0, "a1"),
        (0, "a2"),
        (1, "b1"),
        (0, "a3"),
        (2, "c1"),
        (1, "b2"),
        (2, "c2"),
    ]
    .iter()
    .map(|(id, data)| Unit {
        id: UnitId::new(*id),
        content: UnitContent::String(data.to_string()),
    })
    .collect();

    // Store data in specified order
    for unit in units.iter() {
        client.set_unit(&grouping, unit)?;
    }

    // Revert in input order and check consistency
    for (index, unit) in units.iter().enumerate() {
        let height = INITIAL_HEIGHT + (index as u64);
        let chain_height = ChainHeight::new(height);
        client.revert_by_id(&grouping, &unit.id, &chain_height)?;
        let current_data = client.get_by_id(&grouping, &unit.id)?;
        assert_eq!(current_data, unit.content.to_string());
    }

    Ok(())
}

fn get_units_from_json_property(
    unit_id: &UnitId,
    json_field_names: &(String, String),
    json_field_values: &[(String, f64)],
) -> Vec<Unit> {
    let json_string_vec: Vec<String> = json_field_values
        .iter()
        .map(|values| {
            format!(
                r#"{{"{}": "{}", "{}": {}}}"#,
                json_field_names.0, values.0, json_field_names.1, values.1
            )
        })
        .collect();

    let units: Vec<Unit> = json_string_vec
        .iter()
        .map(|json_string| Unit {
            id: unit_id.clone(),
            content: UnitContent::JsonString(json_string.clone()),
        })
        .collect();

    return units;
}

fn query_db_with_json_property(
    client: &ImmuxDBClient,
    grouping: &GroupingLabel,
    json_field_name: &String,
    json_field_values: &[(String, f64)],
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    let name_properties: Vec<NameProperty> = json_field_values
        .iter()
        .map(|values| {
            (
                PropertyName::from(json_field_name.as_str()),
                UnitContent::Float64(values.1),
            )
        })
        .collect();

    for name_property in &name_properties {
        let (property_name, unit_content) = name_property;
        let res = client
            .get_by_property_name(&grouping, &property_name, &unit_content)
            .unwrap();
        result.push(res);
    }

    return result;
}

fn get_json_value_from_unit(unit: &Unit, key: &String) -> Option<JsonValue> {
    let json_value = serde_json::from_str::<JsonValue>(&unit.content.to_string()).unwrap();
    let json_object = json_value.as_object().unwrap();
    if let Some(data) = json_object.get(key) {
        return Some(data.clone());
    } else {
        return None;
    }
}

#[test]
fn e2e_index_same_unit_id() -> Result<(), Box<dyn Error>> {
    let db_port = 20012;

    thread::spawn(move || launch_db("e2e_index_same_unit_id", db_port));
    notified_sleep(5);

    let client = ImmuxDBClient::new(&format!("localhost:{}", db_port))?;

    let grouping = GroupingLabel::new("GROUPING".as_bytes());
    let chain_name = ChainName::from("immuxtest-index-versioning");

    client.switch_chain(&chain_name)?;

    let unit_id = UnitId::new(1);
    let json_field_names: (String, String) = ("name".to_string(), "age".to_string());
    let json_field_values: [(String, f64); 3] = [
        ("Josh".to_string(), 70 as f64),
        ("David".to_string(), 80 as f64),
        ("Tom".to_string(), 70 as f64),
    ];
    let json_filed_to_be_indexed = &json_field_names.1;

    let units = get_units_from_json_property(&unit_id, &json_field_names, &json_field_values);

    for unit in units.iter() {
        client.set_unit(&grouping, unit)?;
    }

    client.create_index(
        &grouping,
        &PropertyName::from(json_filed_to_be_indexed.as_str()),
    )?;

    let actual_output = query_db_with_json_property(
        &client,
        &grouping,
        &json_filed_to_be_indexed,
        &json_field_values,
    );

    let latest_unit = units.last().unwrap();
    let expected_output: Vec<String> = units
        .iter()
        .map(|unit| {
            let current_unit_index_json_value =
                get_json_value_from_unit(unit, json_filed_to_be_indexed);
            let latest_unit_index_json_value =
                get_json_value_from_unit(latest_unit, json_filed_to_be_indexed);
            if current_unit_index_json_value == latest_unit_index_json_value {
                return latest_unit.content.to_string();
            } else {
                return "".to_string();
            }
        })
        .collect();

    assert_eq!(actual_output, expected_output);

    let new_json_field_values: [(String, f64); 3] = [
        ("Jerry".to_string(), 10 as f64),
        ("Bob".to_string(), 5 as f64),
        ("Bin".to_string(), 10 as f64),
    ];

    let new_units =
        get_units_from_json_property(&unit_id, &json_field_names, &new_json_field_values);

    for unit in new_units.iter() {
        client.set_unit(&grouping, unit)?;
    }

    let actual_output = query_db_with_json_property(
        &client,
        &grouping,
        &json_filed_to_be_indexed,
        &new_json_field_values,
    );

    let latest_unit = new_units.last().unwrap();
    let expected_output: Vec<String> = new_units
        .iter()
        .map(|unit| {
            let current_unit_index_json_value =
                get_json_value_from_unit(unit, json_filed_to_be_indexed);
            let latest_unit_index_json_value =
                get_json_value_from_unit(latest_unit, json_filed_to_be_indexed);
            if current_unit_index_json_value == latest_unit_index_json_value {
                return latest_unit.content.to_string();
            } else {
                return "".to_string();
            }
        })
        .collect();

    assert_eq!(actual_output, expected_output);

    return Ok(());
}

#[test]
fn e2e_index_versioning() -> Result<(), Box<dyn Error>> {
    let db_port = 21119;

    thread::spawn(move || launch_db("e2e_index_versioning", db_port));
    notified_sleep(5);

    let client = ImmuxDBClient::new(&format!("localhost:{}", db_port))?;

    let grouping = GroupingLabel::new("GROUPING".as_bytes());
    let chain_name = ChainName::from("immuxtest-index-versioning");

    client.switch_chain(&chain_name)?;

    let unit_id = UnitId::new(1);
    let json_field_names: (String, String) = ("name".to_string(), "age".to_string());
    let json_field_values: [(String, f64); 6] = [
        ("Josh".to_string(), 70 as f64),
        ("David".to_string(), 80 as f64),
        ("Tom".to_string(), 90 as f64),
        ("Jerry".to_string(), 100 as f64),
        ("Jane".to_string(), 110 as f64),
        ("Bruce".to_string(), 120 as f64),
    ];
    let json_filed_to_be_indexed = &json_field_names.1;

    let units = get_units_from_json_property(&unit_id, &json_field_names, &json_field_values);

    for unit in units.iter() {
        client.set_unit(&grouping, unit)?;
    }

    client.create_index(
        &grouping,
        &PropertyName::from(json_filed_to_be_indexed.as_str()),
    )?;

    let actual_output = query_db_with_json_property(
        &client,
        &grouping,
        &json_filed_to_be_indexed,
        &json_field_values,
    );

    let latest_unit = units.last().unwrap();
    let expected_output: Vec<String> = units
        .iter()
        .map(|unit| {
            let current_unit_index_json_value =
                get_json_value_from_unit(unit, json_filed_to_be_indexed);
            let latest_unit_index_json_value =
                get_json_value_from_unit(latest_unit, json_filed_to_be_indexed);
            if current_unit_index_json_value == latest_unit_index_json_value {
                return latest_unit.content.to_string();
            } else {
                return "".to_string();
            }
        })
        .collect();

    assert_eq!(actual_output, expected_output);

    let target_height = 1;
    let chain_height = ChainHeight::new(target_height);

    client.revert_by_id(&grouping, &unit_id, &chain_height)?;
    let target_unit = &units[target_height as usize - 1];

    let actual_output = query_db_with_json_property(
        &client,
        &grouping,
        &json_filed_to_be_indexed,
        &json_field_values,
    );

    let expected_output: Vec<String> = units
        .iter()
        .map(|unit| {
            let current_unit_index_json_value =
                get_json_value_from_unit(unit, json_filed_to_be_indexed);
            let target_unit_index_json_value =
                get_json_value_from_unit(target_unit, json_filed_to_be_indexed);
            if current_unit_index_json_value == target_unit_index_json_value {
                return target_unit.content.to_string();
            } else {
                return "".to_string();
            }
        })
        .collect();

    assert_eq!(actual_output, expected_output);

    return Ok(());
}
