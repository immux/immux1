use std::net::TcpStream;

use tiny_http::{Header, Method, Request, Response, Server};

use crate::config::ImmuxDBConfiguration;
use crate::cortices::unicus::cortex::HttpParsingError::BodyExtractionError;
use crate::cortices::{Cortex, CortexResponse};
use crate::declarations::commands::{
    Command, InsertCommand, InsertCommandSpec, Outcome, PickChainCommand, SelectCommand,
    SelectCondition,
};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::Answer;
use crate::executor::execute::execute;
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::{config, utils};
use std::collections::HashMap;
use url::Url;

#[derive(Debug)]
pub enum HttpParsingError {
    UrlParsingError,
    BodyParsingError,
    BodyExtractionError,
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
    let (target_collection, target_id) = if segments.len() >= 3 {
        (segments[1], segments[2])
    } else if segments.len() == 2 {
        (segments[1], "")
    } else {
        ("", "")
    };

    match request.method() {
        Method::Get => {
            if let Some(_namespace) = url_info.extract_string_query(config::CHAIN_KEYWORD) {
                let command = Command::NameChain;
                return Ok(command);
            } else if let Some(condition) =
                url_info.extract_string_query(config::SELECT_CONDITION_KEYWORD)
            {
                let command = Command::Select(SelectCommand {
                    grouping: target_collection.as_bytes().to_vec(),
                    condition: SelectCondition::UnconditionalMatch,
                });
                return Ok(command);
            } else {
                let command = Command::Select(SelectCommand {
                    grouping: target_collection.as_bytes().to_vec(),
                    condition: SelectCondition::Id(target_id.as_bytes().to_vec()),
                });
                return Ok(command);
            }
        }
        Method::Put => {
            if let Ok(height) = url_info.extract_numeric_query(config::REVERTALL_QUERY_KEYWORD) {
                return unimplemented!();
            /*
            let instruction = Instruction::AtomicRevertAll(AtomicRevertAllInstruction {
                target_height: height,
            });
            return Ok(instruction);
            */
            } else if let Ok(height) = url_info.extract_numeric_query(config::REVERT_QUERY_KEYWORD)
            {
                return unimplemented!();
            /*
            let instruction = Instruction::AtomicRevert(AtomicRevertInstruction {
                targets: vec![RevertTargetSpec {
                    key: low_key.into_bytes(),
                    height,
                }],
            });
            return Ok(instruction);
            */
            } else if let Some(namespace) = url_info.extract_string_query(config::CHAIN_KEYWORD) {
                let command = Command::PickChain(PickChainCommand {
                    new_chain_name: namespace.as_bytes().to_vec(),
                });
                return Ok(command);
            }

            let command = Command::Insert(InsertCommand {
                grouping: target_collection.as_bytes().to_vec(),
                targets: vec![InsertCommandSpec {
                    id: target_id.as_bytes().to_vec(),
                    value: body.as_bytes().to_vec(),
                }],
            });
            Ok(command)
        }
        _ => Err(HttpParsingError::BodyParsingError.into()),
    }
}

pub fn responder(request: Request, core: &mut ImmuxDBCore) -> ImmuxResult<()> {
    let mut req = request;
    let mut status = 200;
    let mut body = String::new();
    let mut incoming_body = String::new();
    match req.as_reader().read_to_string(&mut incoming_body) {
        Ok(_) => (),
        Err(error) => return Err(HttpParsingError::BodyExtractionError.into()),
    }

    match parse_http_request(&req, &incoming_body) {
        Err(_error) => {
            status = 500;
            body += "request parsing error";
        }
        Ok(command) => match execute(command, core) {
            Err(_error) => status = 500,
            Ok(outcome) => match outcome {
                Outcome::Select(outcome) => {
                    status = 200;
                    for item in outcome.values {
                        body += &utils::utf8_to_string(&item);
                        body += "\r\n";
                    }
                }
                Outcome::NameChain(outcome) => {
                    status = 200;
                    body += &utils::utf8_to_string(&outcome.chain_name);
                }
                Outcome::Insert(outcome) => {
                    status = 200;
                    body += &format!("Inserted {} items", outcome.count);
                }
                _ => {
                    status = 200;
                    body += "Unspecified outcome";
                }
            },
        },
    };
    let response = Response::from_string(body).with_status_code(status);
    req.respond(response);
    return Ok(());
}
