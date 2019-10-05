use std::collections::HashMap;
use std::str::FromStr;

use tiny_http::{Method, Request, Response};
use url::Url;

use crate::config;
use crate::declarations::basics::{
    ChainName, GroupingLabel, PropertyName, UnitContent, UnitContentError, UnitId, UnitIdError,
    UnitSpecifier,
};
use crate::declarations::commands::{
    Command, CreateIndexCommand, InsertCommand, InsertCommandSpec, InspectCommand, Outcome,
    PickChainCommand, RevertAllCommand, RevertCommandTargetSpec, RevertManyCommand, SelectCommand,
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
    UnitContentError(UnitContentError),
}

impl From<UnitIdError> for HttpParsingError {
    fn from(error: UnitIdError) -> Self {
        HttpParsingError::UnitIdError(error)
    }
}

impl From<UnitContentError> for HttpParsingError {
    fn from(error: UnitContentError) -> Self {
        HttpParsingError::UnitContentError(error)
    }
}

pub struct UrlInformation {
    pub queries: HashMap<String, String>,
    pub main_path: String,
}

impl UrlInformation {
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
    let (_host_ip, _chain_name, target_grouping_str, unit_id_str, height_str) = match segments.len()
    {
        5 => (
            segments[0],
            segments[1],
            segments[2],
            segments[3],
            segments[4],
        ),
        4 => (segments[0], segments[1], segments[2], segments[3], ""),
        3 => (segments[0], segments[1], segments[2], "", ""),
        2 => (segments[0], segments[1], "", "", ""),
        1 => (segments[0], "", "", "", ""),
        _ => ("", "", "", "", ""),
    };

    match request.method() {
        Method::Get => {
            if let Some(_) = url_info.extract_string_query(config::CURRENT_CHAIN_KEYWORD) {
                let command = Command::NameChain;
                return Ok(command);
            } else if unit_id_str == config::ALL_KEYWORD {
                let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());
                let command = Command::Select(SelectCommand {
                    grouping: target_grouping,
                    condition: SelectCondition::UnconditionalMatch,
                });
                return Ok(command);
            } else if height_str == config::ALL_KEYWORD {
                let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());
                let target_id = UnitId::read_int_in_str(&unit_id_str)?;
                let command = Command::Inspect(InspectCommand {
                    specifier: UnitSpecifier::new(target_grouping, target_id),
                });
                return Ok(command);
            } else if let (Ok(_unit_id), Ok(_height_int)) = (
                UnitId::read_int_in_str(&unit_id_str),
                height_str.parse::<u64>(),
            ) {
                unimplemented!()
            } else if let Ok(unit_id) = UnitId::read_int_in_str(&unit_id_str) {
                let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());
                let command = Command::Select(SelectCommand {
                    grouping: target_grouping,
                    condition: SelectCondition::Id(unit_id),
                });
                return Ok(command);
            } else if let Some(_) = url_info.extract_string_query(config::BY_KV_KEYWORD) {
                let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());

                for (key, val) in url_info.queries.iter() {
                    if key == config::BY_KV_KEYWORD && val.is_empty() {
                        continue;
                    }
                    let property_name = PropertyName::new(key.as_bytes());
                    let unit_content = UnitContent::from_str(val)?;
                    let command = Command::Select(SelectCommand {
                        grouping: target_grouping,
                        condition: SelectCondition::NameProperty(property_name, unit_content),
                    });
                    return Ok(command);
                }

                return Err(HttpParsingError::UrlParsingError.into());
            } else {
                return Err(HttpParsingError::UrlParsingError.into());
            }
        }
        Method::Put => {
            if let Some(_) = url_info.extract_string_query(config::CURRENT_CHAIN_KEYWORD) {
                let command = Command::PickChain(PickChainCommand {
                    new_chain_name: ChainName::from(body),
                });
                return Ok(command);
            } else if let (Some(_), Some(_), Some(property_name_str)) = (
                url_info.extract_string_query(config::INDICES_KEYWORD),
                url_info.extract_string_query(config::BY_KV_KEYWORD),
                url_info.extract_string_query(config::PROPERTY_NAME_KEYWORD),
            ) {
                let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());
                let command = Command::CreateIndex(CreateIndexCommand {
                    grouping: target_grouping,
                    name: PropertyName::new(property_name_str.as_bytes()),
                });
                return Ok(command);
            } else {
                return Err(HttpParsingError::UrlParsingError.into());
            }
        }
        Method::Post => {
            if unit_id_str == config::LIST_KEYWORD {
                if let Some(height_str) = url_info.extract_string_query(config::REVERT_TO_KEYWORD) {
                    if let Ok(height_u64) = height_str.parse::<u64>() {
                        let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());
                        let height = ChainHeight::new(height_u64);
                        let unit_id_str_vec: Vec<&str> = body.split("\r\n").collect();
                        let res: Result<Vec<UnitId>, UnitIdError> = unit_id_str_vec
                            .iter()
                            .map(|unit_id_str| UnitId::read_int_in_str(unit_id_str))
                            .collect();
                        let specs: Vec<RevertCommandTargetSpec> = res?
                            .iter()
                            .map(|unit_id| {
                                let specifier =
                                    UnitSpecifier::new(target_grouping.clone(), *unit_id);
                                let revert_command_target_spec = RevertCommandTargetSpec {
                                    specifier,
                                    target_height: height,
                                };
                                return revert_command_target_spec;
                            })
                            .collect();

                        let command = Command::RevertMany(RevertManyCommand { specs });
                        return Ok(command);
                    } else {
                        Err(HttpParsingError::UrlParsingError.into())
                    }
                } else {
                    Err(HttpParsingError::UrlParsingError.into())
                }
            } else if unit_id_str == config::ALL_KEYWORD {
                if let Some(height_str) = url_info.extract_string_query(config::REVERT_TO_KEYWORD) {
                    if let Ok(height_u64) = height_str.parse::<u64>() {
                        let height = ChainHeight::new(height_u64);
                        let command = Command::RevertAll(RevertAllCommand {
                            target_height: height,
                        });
                        return Ok(command);
                    } else {
                        Err(HttpParsingError::UrlParsingError.into())
                    }
                } else {
                    Err(HttpParsingError::UrlParsingError.into())
                }
            } else if unit_id_str.is_empty() {
                let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());
                let id_content_str_vec: Vec<&str> = body.split("\r\n").collect();

                let mut targets: Vec<InsertCommandSpec> = vec![];
                let mut index = 0;

                while index < id_content_str_vec.len() - 1 {
                    let id_str = id_content_str_vec[index];
                    let unit_content_str = id_content_str_vec[index + 1];

                    let unit_id = UnitId::read_int_in_str(id_str)?;
                    let unit_content = UnitContent::from_str(unit_content_str)?;

                    let insert_command_spec = InsertCommandSpec {
                        id: unit_id,
                        content: unit_content,
                    };
                    targets.push(insert_command_spec);
                    index += 2;
                }

                let command = Command::Insert(InsertCommand {
                    grouping: target_grouping,
                    targets: targets,
                });
                return Ok(command);
            } else if let Some(height_str) =
                url_info.extract_string_query(config::REVERT_TO_KEYWORD)
            {
                if let Ok(height_u64) = height_str.parse::<u64>() {
                    let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());
                    let height = ChainHeight::new(height_u64 as u64);
                    let target_id = UnitId::read_int_in_str(unit_id_str)?;
                    let specifier = UnitSpecifier::new(target_grouping, target_id);
                    let command = Command::RevertMany(RevertManyCommand {
                        specs: vec![RevertCommandTargetSpec {
                            specifier,
                            target_height: height,
                        }],
                    });
                    return Ok(command);
                } else {
                    Err(HttpParsingError::UrlParsingError.into())
                }
            } else {
                let unit_content = UnitContent::from_str(body)?;
                let target_grouping = GroupingLabel::from(target_grouping_str.as_bytes());
                let target_id = UnitId::read_int_in_str(unit_id_str)?;
                let command = Command::Insert(InsertCommand {
                    grouping: target_grouping,
                    targets: vec![InsertCommandSpec {
                        id: target_id,
                        content: unit_content,
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
