use std::fmt::Formatter;

use libimmuxdb::declarations::basics::{
    ChainName, GroupingLabel, PropertyName, Unit, UnitContent, UnitId,
};
use libimmuxdb::storage::vkv::ChainHeight;
use reqwest;

pub trait ImmuxDBConnector {
    fn get_by_id(&self, grouping: &GroupingLabel, id: &UnitId) -> ClientResult;
    fn get_by_property_name(
        &self,
        grouping: &GroupingLabel,
        property_name: &PropertyName,
        unit_content: &UnitContent,
    ) -> ClientResult;
    fn inspect_by_id(&self, grouping: &GroupingLabel, id: &UnitId) -> ClientResult;
    fn revert_by_id(
        &self,
        grouping: &GroupingLabel,
        id: &UnitId,
        height: &ChainHeight,
    ) -> ClientResult;
    fn set_unit(&self, grouping: &GroupingLabel, unit: &Unit) -> ClientResult;
    fn set_batch_units(&self, grouping: &GroupingLabel, units: &[Unit]) -> ClientResult;
    fn create_index(&self, grouping: &GroupingLabel, property_name: &PropertyName) -> ClientResult;
    fn switch_chain(&self, chain_name: &ChainName) -> ClientResult;
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
    fn get_by_id(&self, grouping: &GroupingLabel, id: &UnitId) -> ClientResult {
        let url = format!(
            "http://{}/{}/{}",
            &self.host,
            grouping.to_string(),
            id.as_int()
        );
        let mut response = reqwest::get(&url)?;
        return response.text().map_err(|e| e.into());
    }

    fn get_by_property_name(
        &self,
        grouping: &GroupingLabel,
        property_name: &PropertyName,
        unit_content: &UnitContent,
    ) -> ClientResult {
        let url = format!(
            "http://{}/{}?select=name_property&{}={}",
            &self.host,
            grouping.to_string(),
            property_name.to_string(),
            unit_content.to_string()
        );
        let mut response = reqwest::get(&url)?;
        return response.text().map_err(|e| e.into());
    }

    fn inspect_by_id(&self, grouping: &GroupingLabel, id: &UnitId) -> ClientResult {
        let mut response = reqwest::get(&format!(
            "http://{}/{}/{}?inspect",
            &self.host,
            grouping.to_string(),
            id.as_int()
        ))?;
        return response.text().map_err(|e| e.into());
    }

    fn revert_by_id(
        &self,
        grouping: &GroupingLabel,
        id: &UnitId,
        height: &ChainHeight,
    ) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!(
                "http://{}/{}/{}?revert={}",
                &self.host,
                grouping.to_string(),
                id.as_int(),
                height.as_u64()
            ))
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn set_unit(&self, grouping: &GroupingLabel, unit: &Unit) -> ClientResult {
        let client = reqwest::Client::new();
        let id = unit.id;
        let property = unit.content.to_string();

        let mut response = client
            .put(&format!(
                "http://{}/{}/{}",
                &self.host,
                grouping.to_string(),
                id.as_int()
            ))
            .body(property)
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn set_batch_units(&self, grouping: &GroupingLabel, units: &[Unit]) -> ClientResult {
        let client = reqwest::Client::new();
        let string_vec: Vec<String> = units
            .iter()
            .map(|unit| -> String { format!("{}|{}", unit.id.as_int(), unit.content.to_string()) })
            .collect();
        let mut response = client
            .put(&format!(
                "http://{}/{}/internal_api_target_id_identifier",
                &self.host,
                grouping.to_string()
            ))
            .body(string_vec.join("\r\n"))
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn create_index(&self, grouping: &GroupingLabel, property_name: &PropertyName) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!(
                "http://{}/{}?index={}",
                &self.host,
                grouping.to_string(),
                property_name.to_string()
            ))
            .send()?;
        return response.text().map_err(|e| e.into());
    }

    fn switch_chain(&self, chain_name: &ChainName) -> ClientResult {
        let client = reqwest::Client::new();
        let mut response = client
            .put(&format!(
                "http://{}/?chain={}",
                &self.host,
                chain_name.to_string()
            ))
            .send()?;
        return response.text().map_err(|e| e.into());
    }
}
