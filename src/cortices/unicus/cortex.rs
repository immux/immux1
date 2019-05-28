use crate::config::ImmuxDBConfiguration;
use crate::cortices::unicus::http::parse_http_request;
use crate::cortices::{Cortex, CortexResponse};
use crate::declarations::errors::ImmuxResult;
use crate::declarations::instructions::Answer;
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::utils;
use std::net::TcpStream;

pub fn unicus_cortex_process_incoming_message(
    bytes: &[u8],
    core: &mut ImmuxDBCore,
    _stream: &TcpStream,
    _config: &ImmuxDBConfiguration,
) -> ImmuxResult<CortexResponse> {
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
            }
            Ok(answer) => match answer {
                Answer::GetOneOk(answer) => http_response += &utils::utf8_to_string(&answer.item),
                Answer::ReadNamespaceOk(answer) => {
                    http_response += &utils::utf8_to_string(&answer.namespace)
                }
                _ => http_response += "success",
            },
        },
    };

    Ok(CortexResponse::Send(http_response.into_bytes()))
}

pub const UNICUS_CORTEX: Cortex = Cortex {
    process_incoming_message: unicus_cortex_process_incoming_message,
    process_first_connection: None,
};
