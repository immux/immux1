use crate::cortices::unicus::http::parse_http_request;
use crate::declarations::errors::{explain_error, UnumResult};
use crate::declarations::instructions::Answer;
use crate::storage::core::{CoreStore, UnumCore};
use crate::utils;

pub fn unicus_cortex_process_incoming_message(
    bytes: &[u8],
    core: &mut UnumCore,
) -> UnumResult<Option<Vec<u8>>> {
    format!("bytes received: {}\n", bytes.len());
    let mut http_response = String::from("HTTP/1.1 200 OK\r\n\r\n");

    let request_string = String::from_utf8_lossy(bytes);

    match parse_http_request(&request_string) {
        Err(_error) => {
            http_response += "request parsing error";
        }
        Ok(instruction) => match core.execute(&instruction) {
            Err(error) => {
                http_response += "instruction execution error";
                http_response += explain_error(error)
            }
            Ok(answer) => match answer {
                Answer::GetOk(answer) => {
                    for item in answer.items {
                        http_response += &utils::utf8_to_string(&item)
                    }
                }
                _ => http_response += "success",
            },
        },
    };

    return Ok(Some(http_response.into_bytes()));
}
