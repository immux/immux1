use crate::cortices::mysql::error::MySQLSerializeError;
use crate::cortices::mysql::handshake_response_41::load_handshake_response;
use crate::cortices::mysql::server_status_flags::{
    load_server_status_flags, serialize_status_flags,
};
use crate::cortices::mysql::utils::{
    serialize_length_encoded_integer, serialize_length_encoded_string,
    u32_to_u8_array_with_length_3,
};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::storage::core::ImmuxDBCore;
use crate::utils::{u16_to_u8_array, utf8_to_string};

pub enum HeaderOption {
    OK = 0x00,
    EOF = 0xfe,
}

pub struct SessionStateInfo {
    pub session_state_change_type: SessionStateChangeType,
    pub data: String,
}

pub enum SessionStateChangeType {
    SessionTrackSystemVariables = 0x00,
    SessionTrackSchema = 0x01,
    SessionTrackStateChange = 0x02,
    SessionTrackGTIDS = 0x03,
}

pub struct OkPacket {
    pub payload_length: u32,
    pub packet_number: u8,
    pub header: HeaderOption,
    pub affected_rows: usize,
    pub last_insert_id: usize,
    pub number_of_warnings: Option<u16>,
    pub info: Option<String>,
    pub session_state_changes: Option<SessionStateInfo>,
}

pub fn serialize_ok_packet(
    ok_packet: OkPacket,
    core: &mut ImmuxDBCore,
    is_connection_phase_ok_packet: bool,
) -> ImmuxResult<Vec<u8>> {
    let handshake_response = load_handshake_response(core)?;
    let mut res = Vec::new();
    res.append(&mut u32_to_u8_array_with_length_3(ok_packet.payload_length)?.to_vec());
    res.push(ok_packet.packet_number);
    res.push(ok_packet.header as u8);
    let mut affected_rows_vec = serialize_length_encoded_integer(ok_packet.affected_rows as u128)?;
    res.append(&mut affected_rows_vec);
    let mut last_insert_id_vec =
        serialize_length_encoded_integer(ok_packet.last_insert_id as u128)?;
    res.append(&mut last_insert_id_vec);

    let status_flags = load_server_status_flags(core)?;

    if handshake_response.capability_flags.client_protocol_41 {
        let status_flags = serialize_status_flags(&status_flags);
        let status_flags_vec = u16_to_u8_array(status_flags);
        res.append(&mut status_flags_vec.to_vec());

        if let Some(number_of_warnings) = ok_packet.number_of_warnings {
            let mut number_of_warnings = u16_to_u8_array(number_of_warnings);
            res.append(&mut number_of_warnings.to_vec())
        } else {
            return Err(ImmuxError::MySQLSerializer(
                MySQLSerializeError::MissingFieldInStruct,
            ));
        }
    } else if handshake_response.capability_flags.client_transactions {
        let status_flags = serialize_status_flags(&status_flags);
        let status_flags_vec = u16_to_u8_array(status_flags);
        res.append(&mut status_flags_vec.to_vec());
    }

    if is_connection_phase_ok_packet {
        return Ok(res);
    } else {
        if handshake_response.capability_flags.client_session_track {
            if let Some(info) = ok_packet.info {
                let mut string_vec = serialize_length_encoded_string(info)?;
                res.append(&mut string_vec);
            } else {
                return Err(ImmuxError::MySQLSerializer(
                    MySQLSerializeError::MissingFieldInStruct,
                ));
            }

            if status_flags.session_state_changed {
                if let Some(session_state_info) = ok_packet.session_state_changes {
                    let mut session_state_info_vec = Vec::new();
                    session_state_info_vec.push(session_state_info.session_state_change_type as u8);
                    session_state_info_vec.append(&mut session_state_info.data.as_bytes().to_vec());
                    let session_state_info_string = utf8_to_string(&session_state_info_vec);
                    res.append(&mut serialize_length_encoded_string(
                        session_state_info_string,
                    )?);
                } else {
                    return Err(ImmuxError::MySQLSerializer(
                        MySQLSerializeError::MissingFieldInStruct,
                    ));
                }
            }
        } else {
            if let Some(info) = ok_packet.info {
                let mut info_vec = info.clone().into_bytes();
                info_vec.push(0x00);
                res.append(&mut info_vec);
            } else {
                return Err(ImmuxError::MySQLSerializer(
                    MySQLSerializeError::MissingFieldInStruct,
                ));
            }
        }

        return Ok(res);
    }
}

#[cfg(test)]
mod ok_packet_tests {

    use crate::config::DEFAULT_CHAIN_NAME;
    use crate::cortices::mysql::handshake_response_41::save_handshake_response;
    use crate::cortices::mysql::ok_packet::{serialize_ok_packet, HeaderOption, OkPacket};
    use crate::cortices::mysql::server_status_flags::save_server_status_flags;
    use crate::storage::core::ImmuxDBCore;
    use crate::storage::kv::KeyValueEngine;

    #[test]
    fn test_serialize_ok_packet() {
        let payload_length = 7;
        let packet_number = 4;
        let header = HeaderOption::OK;
        let affected_rows = 0;
        let last_insert_id = 0;
        let number_of_warnings = Some(0);
        let info = None;
        let session_state_changes = None;

        let ok_packet = OkPacket {
            payload_length,
            packet_number,
            header,
            affected_rows,
            last_insert_id,
            number_of_warnings,
            info,
            session_state_changes,
        };

        let engine_choice = KeyValueEngine::HashMap;
        let mut core = ImmuxDBCore::new(&engine_choice, DEFAULT_CHAIN_NAME.as_bytes()).unwrap();
        let server_status_flags_buffer = [0x02, 0x00];
        let handshake_response_buffer = [
            0xa7, 0x00, 0x00, 0x01, 0x85, 0xa6, 0xff, 0x01, 0x00, 0x00, 0x00, 0x01, 0x2d, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x72, 0x6f, 0x6f, 0x74, 0x00, 0x01,
            0x00, 0x63, 0x61, 0x63, 0x68, 0x69, 0x6e, 0x67, 0x5f, 0x73, 0x68, 0x61, 0x32, 0x5f,
            0x70, 0x61, 0x73, 0x73, 0x77, 0x6f, 0x72, 0x64, 0x00, 0x69, 0x04, 0x5f, 0x70, 0x69,
            0x64, 0x05, 0x39, 0x31, 0x36, 0x39, 0x34, 0x03, 0x5f, 0x6f, 0x73, 0x08, 0x6f, 0x73,
            0x78, 0x31, 0x30, 0x2e, 0x31, 0x34, 0x09, 0x5f, 0x70, 0x6c, 0x61, 0x74, 0x66, 0x6f,
            0x72, 0x6d, 0x06, 0x78, 0x38, 0x36, 0x5f, 0x36, 0x34, 0x0f, 0x5f, 0x63, 0x6c, 0x69,
            0x65, 0x6e, 0x74, 0x5f, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x06, 0x38, 0x2e,
            0x30, 0x2e, 0x31, 0x35, 0x0c, 0x5f, 0x63, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x5f, 0x6e,
            0x61, 0x6d, 0x65, 0x08, 0x6c, 0x69, 0x62, 0x6d, 0x79, 0x73, 0x71, 0x6c, 0x0c, 0x70,
            0x72, 0x6f, 0x67, 0x72, 0x61, 0x6d, 0x5f, 0x6e, 0x61, 0x6d, 0x65, 0x05, 0x6d, 0x79,
            0x73, 0x71, 0x6c,
        ];
        save_server_status_flags(&server_status_flags_buffer, &mut core);
        save_handshake_response(&handshake_response_buffer, &mut core);
        let ok_packet_vec = serialize_ok_packet(ok_packet, &mut core, true).unwrap();
        let buffer = [
            0x07, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        ];
        assert_eq!(buffer.to_vec(), ok_packet_vec);
    }
}
