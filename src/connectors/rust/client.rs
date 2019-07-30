use reqwest;
use std::fmt::Formatter;

pub trait ImmuxDBConnector {
    fn get_by_key(&self, grouping: &str, key: &str) -> ClientResult;
    fn inspect_by_key(&self, grouping: &str, key: &str) -> ClientResult;
    fn revert_by_key(&self, grouping: &str, key: &str, height: u64) -> ClientResult;
    fn set_key_value(&self, collection: &str, key: &str, value: &str) -> ClientResult;
    fn switch_namespace(&self, namespace: &str) -> ClientResult;
}

#[derive(Debug)]
pub enum ImmuxDBClientError {
    Everything,
    Reqewest(reqwest::Error),
}

impl std::fmt::Display for ImmuxDBClientError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        return Ok(());
    }
}

impl std::error::Error for ImmuxDBClientError {
    fn description(&self) -> &str {
        return "ImmuxDB client error";
    }
}

impl std::convert::From<reqwest::Error> for ImmuxDBClientError {
    fn from(error: reqwest::Error) -> ImmuxDBClientError {
        return ImmuxDBClientError::Reqewest(error);
    }
}

pub type ClientResult = Result<String, ImmuxDBClientError>;

pub struct ImmuxDBClient {
    host: String,
}

impl ImmuxDBClient {
    pub fn new(host: &str) -> Result<ImmuxDBClient, ImmuxDBClientError> {
        return Ok(ImmuxDBClient {
            host: host.to_string(),
        });
    }
}

impl ImmuxDBConnector for ImmuxDBClient {
    fn get_by_key(&self, grouping: &str, key: &str) -> ClientResult {
        let mut response = reqwest::get(&format!("http://{}/{}/{}", &self.host, grouping, key))?;
        return response.text().map_err(|e| e.into());
    }

    fn inspect_by_key(&self, grouping: &str, key: &str) -> ClientResult {
        let mut response = reqwest::get(&format!(
            "http://{}/{}/{}?inspect",
            &self.host, grouping, key
        ))?;
        return response.text().map_err(|e| e.into());
    }

    fn revert_by_key(&self, grouping: &str, key: &str, height: u64) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!(
                "http://{}/{}/{}?revert={}",
                &self.host, grouping, key, height
            ))
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn set_key_value(&self, collection: &str, key: &str, value: &str) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!("http://{}/{}/{}", &self.host, collection, key))
            .body(value.to_string())
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn switch_namespace(&self, namespace: &str) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!("http://{}/?chain={}", &self.host, namespace))
            .send()?;
        return response.text().map_err(|e| e.into());
    }
}
