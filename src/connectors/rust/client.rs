use std::fmt::Formatter;

use libimmuxdb::config;
use libimmuxdb::declarations::basics::{ChainName, GroupingLabel, PropertyName, Unit, UnitId};
use libimmuxdb::storage::vkv::ChainHeight;

use reqwest;

pub trait ImmuxDBConnector {
    fn get_by_id(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        id: &UnitId,
    ) -> ClientResult;
    fn inspect_by_id(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        id: &UnitId,
    ) -> ClientResult;
    fn revert_by_id(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        id: &UnitId,
        height: &ChainHeight,
    ) -> ClientResult;
    fn set_unit(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        unit: &Unit,
    ) -> ClientResult;
    fn set_batch_units(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        units: &[Unit],
    ) -> ClientResult;
    fn create_index(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        property_name: &PropertyName,
    ) -> ClientResult;
    fn switch_chain(&self, chain_name: &ChainName) -> ClientResult;
}

#[derive(Debug)]
pub enum ImmuxDBClientError {
    ParameterError,
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
    fn get_by_id(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        id: &UnitId,
    ) -> ClientResult {
        let url = format!(
            "http://{}/{}/{}/{}",
            &self.host,
            chain_name.to_string(),
            grouping.to_string(),
            id.as_int()
        );
        let mut response = reqwest::get(&url)?;
        return response.text().map_err(|e| e.into());
    }

    fn inspect_by_id(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        id: &UnitId,
    ) -> ClientResult {
        let mut response = reqwest::get(&format!(
            "http://{}/{}/{}/{}/{}",
            &self.host,
            chain_name.to_string(),
            grouping.to_string(),
            id.as_int(),
            config::ALL_KEYWORD,
        ))?;
        return response.text().map_err(|e| e.into());
    }

    fn revert_by_id(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        id: &UnitId,
        height: &ChainHeight,
    ) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .post(&format!(
                "http://{}/{}/{}/{}?revert_to={}",
                &self.host,
                chain_name.to_string(),
                grouping.to_string(),
                id.as_int(),
                height.as_u64()
            ))
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn set_unit(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        unit: &Unit,
    ) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .post(&format!(
                "http://{}/{}/{}/{}",
                &self.host,
                chain_name.to_string(),
                grouping.to_string(),
                unit.id.as_int()
            ))
            .body(unit.content.to_string())
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn set_batch_units(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        units: &[Unit],
    ) -> ClientResult {
        let client = reqwest::Client::new();
        let string_vec: Vec<String> = units
            .iter()
            .map(|unit| -> String {
                format!("{}\r\n{}", unit.id.as_int(), unit.content.to_string())
            })
            .collect();

        let mut response = client
            .post(&format!(
                "http://{}/{}/{}",
                &self.host,
                chain_name.to_string(),
                grouping.to_string()
            ))
            .body(string_vec.join("\r\n"))
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn create_index(
        &self,
        chain_name: &ChainName,
        grouping: &GroupingLabel,
        property_name: &PropertyName,
    ) -> ClientResult {
        let client = reqwest::Client::new();

        let mut response = client
            .put(&format!(
                "http://{}/{}/{}?indices&by_kv&{}={}",
                &self.host,
                chain_name.to_string(),
                grouping.to_string(),
                config::PROPERTY_NAME_KEYWORD,
                property_name.to_string(),
            ))
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn switch_chain(&self, chain_name: &ChainName) -> ClientResult {
        let client = reqwest::Client::new();

        let mut response = client
            .put(&format!(
                "http://{}?{}",
                &self.host,
                config::CURRENT_CHAIN_KEYWORD
            ))
            .body(chain_name.to_string())
            .send()?;
        return response.text().map_err(|e| e.into());
    }
}
