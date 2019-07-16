#[cfg(test)]
use reqwest;

type HttpResult = Result<String, reqwest::Error>;

struct DatabaseHttpAccess {
    host: String,
}

impl DatabaseHttpAccess {
    fn new(host: &str) -> DatabaseHttpAccess {
        return DatabaseHttpAccess {
            host: host.to_string(),
        };
    }

    fn get_by_key(&self, grouping: &str, key: &str) -> HttpResult {
        let mut response = reqwest::get(&format!("http://{}/{}/{}", &self.host, grouping, key))?;
        return response.text();
    }

    fn inspect_by_key(&self, grouping: &str, key: &str) -> HttpResult {
        let mut response = reqwest::get(&format!(
            "http://{}/{}/{}?inspect",
            &self.host, grouping, key
        ))?;
        return response.text();
    }

    fn revert_by_key(&self, grouping: &str, key: &str, height: u64) -> HttpResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!(
                "http://{}/{}/{}?revert={}",
                &self.host, grouping, key, height
            ))
            .send()?;
        return response.text();
    }

    fn set_key_value(&self, collection: &str, key: &str, value: &str) -> HttpResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!("http://{}/{}/{}", &self.host, collection, key))
            .body(value.to_string())
            .send()?;
        return response.text();
    }

    fn switch_namespace(&self, namespace: &str) -> HttpResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!("http://{}/?chain={}", &self.host, namespace))
            .send()?;
        return response.text();
    }
}

fn get_db(port: u16) -> DatabaseHttpAccess {
    let host = format!("localhost:{}", port);
    DatabaseHttpAccess::new(&host)
}

#[test]
#[ignore]
fn e2e_change_database_namespace() -> Result<(), reqwest::Error> {
    let db = get_db(1991);

    let id = "doc";
    let grouping = "GROUPING";

    let namespace_a = "immuxtest-ns-A";
    let data_in_a = "data-A";

    let namespace_b = "immuxtest-ns-B";
    let data_in_b = "data-B";

    assert_ne!(namespace_a, namespace_b);
    assert_ne!(data_in_a, data_in_b);

    db.switch_namespace(namespace_a)?;
    db.set_key_value(grouping, id, data_in_a)?;

    db.switch_namespace(namespace_b)?;
    db.set_key_value(grouping, id, data_in_b)?;

    let data_out_b = db.get_by_key(grouping, id)?;
    assert_eq!(data_in_b, data_out_b);

    db.switch_namespace(namespace_a)?;
    let data_out_a = db.get_by_key(grouping, id)?;
    assert_eq!(data_in_a, data_out_a);

    Ok(())
}

const INITIAL_HEIGHT: u64 = 1; // The height 0 is empty; hence first data starts at height 1.

#[test]
#[ignore]
fn e2e_single_document_versioning() -> Result<(), reqwest::Error> {
    let db = get_db(1991);
    db.switch_namespace("immuxtest-single-document-versioning")?;
    let id = "doc_id";
    let grouping = "GROUPING";

    fn dummy_data(height: u64) -> String {
        format!("data-at-height-{}", height)
    }

    let range = INITIAL_HEIGHT..100;

    for height in range.clone() {
        db.set_key_value(grouping, id, &dummy_data(height))?;
    }

    let body_text = db.inspect_by_key(grouping, id)?;
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
        db.revert_by_key(grouping, id, target_height)?;
        let current_value = db.get_by_key(grouping, id)?;
        let expected_value = dummy_data(target_height);
        assert_eq!(current_value, expected_value);
    }

    Ok(())
}

#[test]
#[ignore]
fn e2e_multiple_document_versioning() -> Result<(), reqwest::Error> {
    let db = get_db(1991);
    db.switch_namespace("immuxtest-multiple-document-versioning")?;

    let grouping = "GROUPING";

    let inputs: Vec<(&str, &str)> = vec![
        //id, data
        ("A", "a1"),
        ("A", "a2"),
        ("B", "b1"),
        ("A", "a3"),
        ("C", "c1"),
        ("B", "b2"),
        ("C", "c2"),
    ];

    // Store data in specified order
    for input in inputs.iter() {
        let (id, data) = input;
        db.set_key_value(grouping, id, data)?;
    }

    // Revert in input order and check consistency
    for (index, input) in inputs.iter().enumerate() {
        let height = INITIAL_HEIGHT + (index as u64);
        let (id, input_data) = input;
        db.revert_by_key(grouping, id, height)?;
        let current_data = db.get_by_key(grouping, id)?;
        assert_eq!(&current_data, input_data);
    }

    Ok(())
}
