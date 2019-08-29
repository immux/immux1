use std::fmt::Formatter;

use reqwest;

pub trait ImmuxDBConnector {
    fn get_by_id(&self, grouping: &str, id: u128) -> ClientResult;
    fn inspect_by_id(&self, grouping: &str, id: u128) -> ClientResult;
    fn revert_by_id(&self, grouping: &str, id: u128, height: u64) -> ClientResult;
    fn set_by_id(&self, collection: &str, id: u128, value: &str) -> ClientResult;
    fn switch_namespace(&self, namespace: &str) -> ClientResult;
}

#[derive(Debug)]
pub enum ImmuxDBClientError {
    Everything,
    Reqwest(reqwest::Error),
}

impl std::fmt::Display for ImmuxDBClientError {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), std::fmt::Error> {
        return Ok(());
    }
}

impl std::error::Error for ImmuxDBClientError {
    fn description(&self) -> &str {
        return "ImmuxDB client error";
    }
}

impl From<reqwest::Error> for ImmuxDBClientError {
    fn from(error: reqwest::Error) -> ImmuxDBClientError {
        return ImmuxDBClientError::Reqwest(error);
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
    fn get_by_id(&self, grouping: &str, id: u128) -> ClientResult {
        let url = format!("http://{}/{}/{}", &self.host, grouping, id);
        let mut response = reqwest::get(&url)?;
        return response.text().map_err(|e| e.into());
    }

    fn inspect_by_id(&self, grouping: &str, id: u128) -> ClientResult {
        let mut response = reqwest::get(&format!(
            "http://{}/{}/{}?inspect",
            &self.host, grouping, id
        ))?;
        return response.text().map_err(|e| e.into());
    }

    fn revert_by_id(&self, grouping: &str, id: u128, height: u64) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!(
                "http://{}/{}/{}?revert={}",
                &self.host, grouping, id, height
            ))
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn set_by_id(&self, collection: &str, id: u128, value: &str) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!("http://{}/{}/{}", &self.host, collection, id))
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
