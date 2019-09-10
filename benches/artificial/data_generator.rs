use immuxdb_bench_utils::UnitList;
use libimmuxdb::declarations::basics::{Unit, UnitContent, UnitId};
use serde_json::{json, Number, Value};

use crate::bench::{PropertySpec, UnitIdSpec};

pub fn get_string_with_fix_size(size: usize, pattern: char) -> String {
    let mut res: String = "".to_string();
    while res.len() < size {
        res.push(pattern);
    }
    return res;
}

pub fn generate_json_table(
    row_count: usize,
    unit_id_spec: &UnitIdSpec,
    json_spec: &[PropertySpec],
) -> UnitList {
    let mut res = vec![];

    for id_int in 0..(row_count as u64) {
        let id = unit_id_spec(id_int);
        let json = get_json(json_spec, &UnitId::new(id_int as u128), row_count);
        let content = UnitContent::JsonString(json.to_string());
        let unit = Unit { id, content };
        res.push(unit);
    }

    return res;
}

fn get_json(json_spec: &[PropertySpec], id: &UnitId, row_count: usize) -> Value {
    let mut json = json!({});

    for property_spec in json_spec {
        let (property_name, unit_content) = property_spec(id, row_count);

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
