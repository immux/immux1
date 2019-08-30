use crate::{PropertySpec, UnitIDSpec};
use immuxdb_bench_utils::JsonTable;
use libimmuxdb::declarations::basics::{PropertyName, Unit, UnitContent, UnitId};
use std::collections::HashSet;

use serde_json::{json, Number, Value};

pub fn get_string_with_fix_size(size: usize, pattern: char) -> String {
    let mut res: String = "".to_string();
    while res.len() < size {
        res.push(pattern);
    }
    return res;
}

pub fn generate_json_table(
    row_count: &usize,
    unit_id_spec: &UnitIDSpec,
    json_spec: &Vec<PropertySpec>,
) -> (JsonTable, Vec<PropertyName>) {
    let mut res = vec![];
    let mut property_name_set = HashSet::new();

    for id_int in 0..(*row_count as u64) {
        let id = unit_id_spec(id_int);
        let json = generate_json(
            json_spec,
            &UnitId::new(id_int as u128),
            *row_count,
            &mut property_name_set,
        );
        let content = UnitContent::JsonString(json.to_string());
        let unit = Unit { id, content };
        res.push(unit);
    }

    let property_name_vec: Vec<PropertyName> = property_name_set
        .iter()
        .map(|property_name| return property_name.to_owned())
        .collect();
    return (res, property_name_vec);
}

fn generate_json(
    json_spec: &Vec<PropertySpec>,
    id: &UnitId,
    row_count: usize,
    property_name_set: &mut HashSet<PropertyName>,
) -> Value {
    let mut json = json!({});

    for property_spec in json_spec {
        let (property_name, unit_content) = (property_spec)(id, row_count);
        property_name_set.insert(property_name.clone());
        match unit_content {
            UnitContent::String(string) => {
                json[property_name.to_string()] = Value::String(string);
            }
            UnitContent::Bool(boolean) => {
                json[property_name.to_string()] = Value::Bool(boolean);
            }
            UnitContent::Float64(number_f64) => {
                let number = Number::from_f64(number_f64).unwrap();
                json[property_name.to_string()] = Value::Number(number);
            }
            _ => {}
        }
    }

    return json;
}
