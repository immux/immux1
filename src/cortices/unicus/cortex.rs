use std::collections::HashMap;
use std::convert::TryFrom;

use tiny_http::{Method, Request, Response};
use url::Url;

use crate::config;
use crate::declarations::basics::{
    ChainName, GroupingLabel, PropertyName, UnitContent, UnitId, UnitIdError, UnitSpecifier,
};
use crate::declarations::commands::{
    Command, CreateIndexCommand, InsertCommand, InsertCommandSpec, InspectCommand, Outcome,
    PickChainCommand, RevertAllCommand, RevertCommand, RevertCommandTargetSpec, SelectCommand,
    SelectCondition,
};
use crate::declarations::errors::ImmuxError::HttpResponse;
use crate::declarations::errors::ImmuxResult;
use crate::executor::execute::execute;
use crate::storage::core::ImmuxDBCore;
use crate::storage::vkv::ChainHeight;

#[derive(Debug)]
pub enum HttpParsingError {
    UrlParsingError,
    BodyParsingError,
    BodyExtractionError,
    UnitIdError(UnitIdError),
}

impl From<UnitIdError> for HttpParsingError {
    fn from(error: UnitIdError) -> Self {
        HttpParsingError::UnitIdError(error)
    }
}

pub struct UrlInformation {
    pub queries: HashMap<String, String>,
    pub main_path: String,
}

impl UrlInformation {
    fn extract_numeric_query(&self, key: &str) -> Result<u64, HttpParsingError> {
        match self.queries.get(key) {
            None => Err(HttpParsingError::UrlParsingError),
            Some(string) => match string.parse::<u64>() {
                Err(_error) => Err(HttpParsingError::UrlParsingError),
                Ok(value) => Ok(value),
            },
        }
    }
    fn extract_string_query(&self, key: &str) -> Option<String> {
        match self.queries.get(key) {
            None => None,
            Some(string) => Some(string.clone()),
        }
    }
}

pub fn parse_path(path: &str) -> Result<UrlInformation, HttpParsingError> {
    let path_to_parse = format!("{}{}", "http://127.0.0.1", path);
    match Url::parse(&path_to_parse) {
        Err(_error) => Err(HttpParsingError::UrlParsingError.into()),
        Ok(parse) => {
            let url_queries: HashMap<_, _> = parse.query_pairs().into_owned().collect();
            Ok(UrlInformation {
                queries: url_queries,
                main_path: String::from(parse.path()),
            })
        }
    }
}

fn parse_http_request(request: &Request, body: &str) -> Result<Command, HttpParsingError> {
    let url_info = parse_path(&request.url())?;

    let segments: Vec<&str> = url_info.main_path.split("/").collect();
    let (target_grouping_str, target_id_str) = if segments.len() >= 3 {
        (segments[1], segments[2])
    } else if segments.len() == 2 {
        (segments[1], "")
    } else {
        ("", "")
    };

    let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());

    match request.method() {
        Method::Get => {
            if let Some(_namespace) = url_info.extract_string_query(config::CHAIN_KEYWORD) {
                let command = Command::NameChain;
                return Ok(command);
            } else if let Some(_condition) =
                url_info.extract_string_query(config::SELECT_CONDITION_KEYWORD)
            {
                let command = Command::Select(SelectCommand {
                    grouping: target_grouping,
                    condition: SelectCondition::UnconditionalMatch,
                });
                return Ok(command);
            } else if let Some(_) = url_info.extract_string_query(config::INSPECT_KEYWORD) {
                let target_id = UnitId::try_from(target_id_str)?;
                let command = Command::Inspect(InspectCommand {
                    specifier: UnitSpecifier::new(target_grouping, target_id),
                });
                return Ok(command);
            } else {
                let target_id = UnitId::try_from(target_id_str)?;
                let command = Command::Select(SelectCommand {
                    grouping: target_grouping,
                    condition: SelectCondition::Id(target_id),
                });
                return Ok(command);
            }
        }
        Method::Put => {
            if let Ok(height_u64) = url_info.extract_numeric_query(config::REVERT_QUERY_KEYWORD) {
                let height = ChainHeight::new(height_u64);
                let target_id = UnitId::try_from(target_id_str)?;
                let specifier = UnitSpecifier::new(target_grouping, target_id);
                let command = Command::RevertOne(RevertCommand {
                    specs: vec![RevertCommandTargetSpec {
                        specifier,
                        target_height: height,
                    }],
                });
                return Ok(command);
            } else if let Ok(height_u64) =
                url_info.extract_numeric_query(config::REVERTALL_QUERY_KEYWORD)
            {
                let height = ChainHeight::new(height_u64);
                let command = Command::RevertAll(RevertAllCommand {
                    target_height: height,
                });
                return Ok(command);
            } else if let Some(namespace) = url_info.extract_string_query(config::CHAIN_KEYWORD) {
                let command = Command::PickChain(PickChainCommand {
                    new_chain_name: ChainName::from(namespace.as_str()),
                });
                return Ok(command);
            } else if let Some(property_name_str) =
                url_info.extract_string_query(config::CREATE_INDEX_KEYWORD)
            {
                let command = Command::CreateIndex(CreateIndexCommand {
                    grouping: target_grouping,
                    name: PropertyName::new(property_name_str.as_bytes()),
                });
                return Ok(command);
            } else if target_id_str == config::INTERNAL_API_TARGET_ID_IDENTIFIER {
                //                this is an internal API
                let mut targets: Vec<InsertCommandSpec> = vec![];

                let units_string_vec: Vec<&str> = body.split("\r\n").collect();
                for unit_str in units_string_vec {
                    let id_property: Vec<&str> = unit_str.split("|").collect();
                    let id_str = id_property[0];
                    let property_str = id_property[1];
                    let insert_command_spec = InsertCommandSpec {
                        id: UnitId::try_from(id_str)?,
                        content: UnitContent::JsonString(property_str.to_string()),
                    };

                    targets.push(insert_command_spec);
                }
                let command = Command::Insert(InsertCommand {
                    grouping: target_grouping,
                    targets,
                });
                return Ok(command);
            } else {
                let target_id = UnitId::try_from(target_id_str)?;
                let command = Command::Insert(InsertCommand {
                    grouping: target_grouping,
                    targets: vec![InsertCommandSpec {
                        id: target_id,
                        content: UnitContent::JsonString(body.to_string()),
                    }],
                });
                return Ok(command);
            }
        }
        _ => Err(HttpParsingError::BodyParsingError.into()),
    }
}

pub fn responder(request: Request, core: &mut ImmuxDBCore) -> ImmuxResult<()> {
    let mut req = request;
    let mut incoming_body = String::new();
    match req.as_reader().read_to_string(&mut incoming_body) {
        Ok(_) => (),
        Err(_error) => return Err(HttpParsingError::BodyExtractionError.into()),
    }

    let (status, body): (u16, String) = match parse_http_request(&req, &incoming_body) {
        Err(error) => (500, format!("request parsing error {:?}", error)),
        Ok(command) => match execute(command, core) {
            Err(error) => (500, format!("executing error {:?}", error)),
            Ok(outcome) => match outcome {
                Outcome::Select(outcome) => {
                    let mut body = String::new();
                    let should_break_line = outcome.units.len() >= 2;
                    for unit in outcome.units {
                        body += &unit.content.to_string();
                        if should_break_line {
                            body += "\r\n";
                        }
                    }
                    (200, body)
                }
                Outcome::NameChain(outcome) => (200, outcome.chain_name.to_string()),
                Outcome::Insert(outcome) => (200, format!("Inserted {} items", outcome.count)),
                Outcome::Inspect(outcome) => {
                    let mut body = String::new();
                    for inspection in outcome.inspections {
                        body += &inspection.to_string();
                        body += "\r\n";
                    }
                    (200, body)
                }
                _ => (200, String::from("Unspecified outcome")),
            },
        },
    };
    let response = Response::from_string(body).with_status_code(status);
    match req.respond(response) {
        Ok(_) => {
            return Ok(());
        }
        Err(error) => {
            return Err(HttpResponse(error));
        }
    }
}
