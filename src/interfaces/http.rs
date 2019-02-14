use crate::config;
use crate::interfaces::queries::*;
use std::collections::HashMap;
use url::{Host, Url};

fn get_html_body(s: &str) -> &str {
    let newline_pos = s.find("\r\n\r\n");
    match newline_pos {
        None => "",
        Some(pos) => &s[pos..],
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
    let (method, path, version) = (
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

pub fn parse_path(path: &str) -> UrlInformation {
    let path_to_parse = format!("{}{}", "http://127.0.0.1", path);
    let url_parse = Url::parse(&path_to_parse).unwrap();
    let url_queries: HashMap<_, _> = url_parse.query_pairs().into_owned().collect();

    UrlInformation {
        queries: url_queries,
        main_path: String::from(url_parse.path()),
    }
}

pub fn parse_http_request(body: &str) -> Option<Query> {
    let (method, full_path) = parse_request_line(body);
    let url_info = parse_path(&full_path);

    let segments: Vec<&str> = url_info.main_path.split("/").collect();
    if segments.len() <= 2 {
        return None;
    }
    let (target_collection, target_id) = (segments[1], segments[2]);
    let low_key = get_lowlevel_key(target_collection, target_id);

    match method.as_str() {
        "GET" => {
            let low_key = get_lowlevel_key(target_collection, target_id);
            if let Some(height) = url_info.queries.get(config::HEIGHT_QUERY_KEYWORD) {
                let query = GetKeyAtHeightQuery {
                    key: low_key.into_bytes(),
                    height: height.parse::<u64>().unwrap(),
                };
                Some(Query::GetKeyAtHeight(query))
            } else {
                let query = GetKeyQuery {
                    key: low_key.into_bytes(),
                };
                Some(Query::GetKey(query))
            }
        }
        "PUT" => {
            let low_key = get_lowlevel_key(target_collection, target_id);
            if let Some(height) = url_info.queries.get(config::REVERT_QUERY_KEYWORD) {
                let query = RevertByKeyQuery {
                    key: low_key.into_bytes(),
                    height: height.parse::<u64>().unwrap(),
                };
                return Some(Query::RevertByKey(query));
            }
            let value = get_html_body(&body);
            let query = SetKeyQuery {
                key: low_key.into_bytes(),
                value: value.as_bytes().to_vec(),
            };
            Some(Query::SetKey(query))
        }
        _ => unimplemented!(),
    }
}
