use crate::cortices::mysql::capability_flags::{parse_capability_flags, CapabilityFlags};
use crate::cortices::mysql::character_set::{parse_character_set, CharacterSet};
use crate::cortices::mysql::error::{MySQLParserError, MySQLSerializeError};
use crate::cortices::mysql::utils::{
    parse_length_encoded_integer, parse_string_with_fixed_length, parse_u32_with_length_3,
};
use crate::cortices::utils::{parse_cstring, parse_u32, parse_u8};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::declarations::instructions::{
    Answer, AtomicGetInstruction, AtomicSetInstruction, GetTargetSpec, Instruction, SetTargetSpec,
};
use crate::storage::core::{CoreStore, UnumCore};
use std::collections::HashMap;

pub struct HandshakeResponse {
    pub payload_length: u32,
    pub packet_number: u8,
    pub capability_flags: CapabilityFlags,
    pub max_packet_size: u32,
    pub character_set: CharacterSet,
    pub user_name: String,
    pub auth_response: Option<String>,
    pub database: Option<String>,
    pub auth_plugin_name: Option<String>,
    pub attribute: Option<HashMap<String, String>>,
}

const MYSQL_HANDSHAKE_RESPONSE_KEY: &str = "_MYSQL_HANDSHAKE_RESPONSE";

pub fn save_handshake_response(buffer: &[u8], core: &mut UnumCore) -> UnumResult<()> {
    let instruction = AtomicSetInstruction {
        targets: vec![SetTargetSpec {
            key: MYSQL_HANDSHAKE_RESPONSE_KEY.as_bytes().to_vec(),
            value: buffer.to_vec(),
        }],
    };

    match core.execute(&Instruction::AtomicSet(instruction)) {
        Err(_error) => Err(UnumError::MySQLParser(
            MySQLParserError::CannotSetClientStatus,
        )),
        Ok(_) => Ok(()),
    }
}

pub fn load_handshake_response(core: &mut UnumCore) -> UnumResult<HandshakeResponse> {
    let instruction = AtomicGetInstruction {
        targets: vec![GetTargetSpec {
            key: MYSQL_HANDSHAKE_RESPONSE_KEY.as_bytes().to_vec(),
            height: None,
        }],
    };
    match core.execute(&Instruction::AtomicGet(instruction)) {
        Err(_error) => {
            return Err(UnumError::MySQLSerializer(
                MySQLSerializeError::CannotReadClientStatus,
            ));
        }
        Ok(answer) => match answer {
            Answer::GetOk(get_answer) => {
                let target = &get_answer.items[0];
                let res = parse_handshake_response(target)?;
                return Ok(res);
            }
            _ => {
                return Err(UnumError::MySQLSerializer(
                    MySQLSerializeError::CannotReadClientStatus,
                ));
            }
        },
    }
}

pub fn parse_handshake_response(buffer: &[u8]) -> UnumResult<HandshakeResponse> {
    let mut index: usize = 0;
    let (payload_length, offset) = parse_u32_with_length_3(&buffer[index..])?;
    index += offset;
    let (packet_number, offset) = parse_u8(&buffer[index..])?;
    index += offset;
    let (capability_flags, offset) = parse_capability_flags(&buffer[index..])?;
    index += offset;
    let (max_packet_size, offset) = parse_u32(&buffer[index..])?;
    index += offset;
    let (character_set, offset) = parse_character_set(&buffer[index..])?;
    index += offset;
    let reserved_bytes_length = 23;
    index += reserved_bytes_length;
    let (user_name, offset) = parse_cstring(&buffer[index..])?;
    index += offset;

    let mut auth_response = None;
    if capability_flags.client_plugin_auth_lenenc_client_data {
        let (string_length, offset) = parse_length_encoded_integer(&buffer[index..])?;
        index += offset;
        let (val, offset) = parse_string_with_fixed_length(&buffer[index..], string_length)?;
        auth_response = Some(val);
        index += offset;
    } else if capability_flags.client_secure_connection {
        index += 1;
        let (val, offset) = parse_string_with_fixed_length(&buffer[index..], 1)?;
        auth_response = Some(val);
        index += offset;
    } else {
        let (val, offset) = parse_cstring(&buffer[index..])?;
        auth_response = Some(val);
        index += offset;
    }

    let mut database = None;
    if capability_flags.client_connect_with_db {
        let (val, offset) = parse_cstring(&buffer[index..])?;
        database = Some(val);
        index += offset;
    }

    let mut auth_plugin_name = None;
    if capability_flags.client_plugin_auth {
        let (val, offset) = parse_cstring(&buffer[index..])?;
        auth_plugin_name = Some(val);
        index += offset;
    }

    let mut attribute_hash_map = HashMap::new();
    if capability_flags.client_connect_attrs {
        let (hash_map_length, offset) = parse_length_encoded_integer(&buffer[index..])?;
        index += offset;

        let mut current_length = 0;

        // TODO: Here we assume the input buffer from official MySQL client is correct, #75
        loop {
            if current_length == hash_map_length {
                break;
            }
            let (key_length, offset) = parse_length_encoded_integer(&buffer[index..])?;
            index += offset;
            current_length += offset;
            let (key, offset) = parse_string_with_fixed_length(&buffer[index..], key_length)?;
            index += offset;
            current_length += offset;

            let (val_length, offset) = parse_length_encoded_integer(&buffer[index..])?;
            index += offset;
            current_length += offset;
            let (val, offset) = parse_string_with_fixed_length(&buffer[index..], val_length)?;
            index += offset;
            current_length += offset;

            attribute_hash_map.insert(key, val);
        }

        if index != buffer.len() {
            return Err(UnumError::MySQLParser(MySQLParserError::InputBufferError));
        }
    }

    let attribute = if attribute_hash_map.len() != 0 {
        Some(attribute_hash_map)
    } else {
        None
    };

    let handshake_response = HandshakeResponse {
        payload_length,
        packet_number,
        capability_flags,
        max_packet_size,
        character_set,
        user_name,
        auth_response,
        database,
        auth_plugin_name,
        attribute,
    };

    return Ok(handshake_response);
}

#[cfg(test)]
mod handshake_response_41_tests {

    use crate::cortices::mysql::handshake_response_41::parse_handshake_response;

    static HANDSHAKE_RESPONSE_FIXTURE: [u8; 171] = [
        0xa7, 0x00, 0x00, 0x01, 0x85, 0xa6, 0xff, 0x01, 0x00, 0x00, 0x00, 0x01, 0x2d, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x72, 0x6f, 0x6f, 0x74, 0x00, 0x01, 0x00, 0x63, 0x61,
        0x63, 0x68, 0x69, 0x6e, 0x67, 0x5f, 0x73, 0x68, 0x61, 0x32, 0x5f, 0x70, 0x61, 0x73, 0x73,
        0x77, 0x6f, 0x72, 0x64, 0x00, 0x69, 0x04, 0x5f, 0x70, 0x69, 0x64, 0x05, 0x39, 0x31, 0x36,
        0x39, 0x34, 0x03, 0x5f, 0x6f, 0x73, 0x08, 0x6f, 0x73, 0x78, 0x31, 0x30, 0x2e, 0x31, 0x34,
        0x09, 0x5f, 0x70, 0x6c, 0x61, 0x74, 0x66, 0x6f, 0x72, 0x6d, 0x06, 0x78, 0x38, 0x36, 0x5f,
        0x36, 0x34, 0x0f, 0x5f, 0x63, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x5f, 0x76, 0x65, 0x72, 0x73,
        0x69, 0x6f, 0x6e, 0x06, 0x38, 0x2e, 0x30, 0x2e, 0x31, 0x35, 0x0c, 0x5f, 0x63, 0x6c, 0x69,
        0x65, 0x6e, 0x74, 0x5f, 0x6e, 0x61, 0x6d, 0x65, 0x08, 0x6c, 0x69, 0x62, 0x6d, 0x79, 0x73,
        0x71, 0x6c, 0x0c, 0x70, 0x72, 0x6f, 0x67, 0x72, 0x61, 0x6d, 0x5f, 0x6e, 0x61, 0x6d, 0x65,
        0x05, 0x6d, 0x79, 0x73, 0x71, 0x6c,
    ];

    #[test]
    fn test_parse_handshake_response() {
        let buffer = HANDSHAKE_RESPONSE_FIXTURE;
        let handshake_response = parse_handshake_response(&buffer).unwrap();
        assert_eq!(handshake_response.payload_length, 167);
        assert_eq!(handshake_response.packet_number, 1);
        assert_eq!(handshake_response.max_packet_size, 16777216);
        assert_eq!(handshake_response.user_name, "root");
        assert_eq!(
            handshake_response.auth_plugin_name.unwrap(),
            "caching_sha2_password"
        );

        let attribute = handshake_response.attribute.to_owned().unwrap();
        assert!(attribute.contains_key("_pid"));
        assert!(attribute.contains_key("_os"));
        assert!(attribute.contains_key("_platform"));
        assert!(attribute.contains_key("_client_version"));
        assert!(attribute.contains_key("_client_name"));
        assert!(attribute.contains_key("program_name"));

        assert_eq!(attribute.get("_pid").unwrap(), "91694");
        assert_eq!(attribute.get("_os").unwrap(), "osx10.14");
        assert_eq!(attribute.get("_platform").unwrap(), "x86_64");
        assert_eq!(attribute.get("_client_version").unwrap(), "8.0.15");
        assert_eq!(attribute.get("_client_name").unwrap(), "libmysql");
        assert_eq!(attribute.get("program_name").unwrap(), "mysql");
    }
}