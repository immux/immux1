use crate::config;
use crate::declarations::errors::UnumError;
use crate::declarations::instructions::{
    GetInstruction, GetTargetSpec, Instruction, RevertAllInstruction, RevertInstruction,
    RevertTargetSpec, SetInstruction, SetTargetSpec,
};
use std::collections::HashMap;
use url::Url;

fn get_html_body(s: &str) -> &str {
    let newline_pos = s.find("\r\n\r\n");
    match newline_pos {
        None => "",
        Some(pos) => &s[pos + 4..], // 4 == "\r\n\r\n".length()
    }
}

fn get_lowlevel_key(target_collection: &str, target_id: &str) -> String {
    [target_collection, target_id].join(".")
}

pub fn parse_request_line(body: &str) -> (String, String) {
    let split = body.split("\n");
    let lines: Vec<&str> = split.collect();
    let first_line = lines[0];
    let first_line_string = first_line.to_string();
    let split = first_line_string.split(" ");
    let first_line_words: Vec<&str> = split.collect();
    if first_line_words.len() <= 2 {
        return (String::from(""), String::from(""));
    }
    let (method, path, _version) = (
        first_line_words[0],
        first_line_words[1],
        first_line_words[2],
    );
    (String::from(method), String::from(path))
}

pub struct UrlInformation {
    pub queries: HashMap<String, String>,
    pub main_path: String,
}

impl UrlInformation {
    fn extract_numeric_query(&self, key: &str) -> Result<u64, UnumError> {
        match self.queries.get(key) {
            None => Err(UnumError::UrlParseError),
            Some(string) => match string.parse::<u64>() {
                Err(_error) => Err(UnumError::UrlParseError),
                Ok(value) => Ok(value),
            },
        }
    }
}

pub fn parse_path(path: &str) -> Result<UrlInformation, UnumError> {
    let path_to_parse = format!("{}{}", "http://127.0.0.1", path);
    match Url::parse(&path_to_parse) {
        Err(_error) => Err(UnumError::UrlParseError),
        Ok(parse) => {
            let url_queries: HashMap<_, _> = parse.query_pairs().into_owned().collect();
            Ok(UrlInformation {
                queries: url_queries,
                main_path: String::from(parse.path()),
            })
        }
    }
}

pub fn parse_http_request(message: &str) -> Result<Instruction, UnumError> {
    let (method, full_path) = parse_request_line(message);
    let url_info = parse_path(&full_path)?;

    let segments: Vec<&str> = url_info.main_path.split("/").collect();
    let low_key = if segments.len() >= 3 {
        let (target_collection, target_id) = (segments[1], segments[2]);
        get_lowlevel_key(target_collection, target_id)
    } else {
        String::from("")
    };

    match method.as_str() {
        "GET" => {
            let instruction = Instruction::Get(GetInstruction {
                targets: vec![GetTargetSpec {
                    key: low_key.into_bytes(),
                    height: if let Ok(height) =
                        url_info.extract_numeric_query(config::HEIGHT_QUERY_KEYWORD)
                    {
                        Some(height)
                    } else {
                        None
                    },
                }],
            });
            return Ok(instruction);
        }
        "PUT" => {
            if let Ok(height) = url_info.extract_numeric_query(config::REVERTALL_QUERY_KEYWORD) {
                let instruction = Instruction::RevertAll(RevertAllInstruction {
                    target_height: height,
                });
                return Ok(instruction);
            } else if let Ok(height) = url_info.extract_numeric_query(config::REVERT_QUERY_KEYWORD)
            {
                let instruction = Instruction::Revert(RevertInstruction {
                    targets: vec![RevertTargetSpec {
                        key: low_key.into_bytes(),
                        height,
                    }],
                });
                return Ok(instruction);
            }
            let value = get_html_body(&message);
            let instruction = Instruction::Set(SetInstruction {
                targets: vec![SetTargetSpec {
                    key: low_key.into_bytes(),
                    value: value.as_bytes().to_vec(),
                }],
            });
            Ok(instruction)
        }
        _ => unimplemented!(),
    }
}
