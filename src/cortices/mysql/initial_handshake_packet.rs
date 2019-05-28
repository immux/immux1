use crate::cortices::mysql::capability_flags::{serialize_capability_flags, CapabilityFlags};
use crate::cortices::mysql::character_set::get_character_set_value;
use crate::cortices::mysql::character_set::CharacterSet;
use crate::cortices::mysql::error::MySQLSerializeError;
use crate::cortices::mysql::server_status_flags::{serialize_status_flags, ServerStatusFlags};
use crate::cortices::mysql::utils::{u32_to_u8_array_with_length_3, MYSQL_PACKET_HEADER_LENGTH};
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::utils::{u16_to_u8_array, u32_to_u8_array};

pub struct InitialHandshakePacket {
    pub payload_length: u32,
    pub packet_number: u8,
    pub protocol_version: u8,
    pub server_version: String,
    pub connection_id: u32,
    pub auth_plugin_data: String,
    pub capability_flags: CapabilityFlags,
    pub character_set: Option<CharacterSet>,
    pub status_flags: Option<ServerStatusFlags>,
    pub auth_plugin_name: String,
}

pub fn serialize_auth_plugin_data(auth_plugin_data: String) -> ImmuxResult<(Vec<u8>, Vec<u8>)> {
    let auth_plugin_data_vec = auth_plugin_data.into_bytes();
    if auth_plugin_data_vec.is_empty() {
        return Err(ImmuxError::MySQLSerializer(
            MySQLSerializeError::SerializeAuthPluginDataError,
        ));
    }
    let mut auth_plugin_data_part1 = Vec::new();
    let mut auth_plugin_data_part2 = Vec::new();
    if auth_plugin_data_vec.len() <= 8 {
        auth_plugin_data_part1 = auth_plugin_data_vec[0..].to_vec();
        let padding_size = 8 - auth_plugin_data_vec.len();
        for _i in 0..padding_size {
            auth_plugin_data_part1.push(0x00);
        }
    } else if auth_plugin_data_vec.len() <= 20 {
        auth_plugin_data_part1 = auth_plugin_data_vec[0..8].to_vec();
        auth_plugin_data_part2 = auth_plugin_data_vec[8..].to_vec();
    } else {
        auth_plugin_data_part1 = auth_plugin_data_vec[0..8].to_vec();
        auth_plugin_data_part2 = auth_plugin_data_vec[8..20].to_vec();
    }

    auth_plugin_data_part1.push(0x00);
    auth_plugin_data_part2.push(0x00);

    return Ok((auth_plugin_data_part1, auth_plugin_data_part2));
}

pub fn serialize_initial_handshake_packet(
    initial_handshake_packet: InitialHandshakePacket,
) -> ImmuxResult<Vec<u8>> {
    let mut res = Vec::new();
    res.append(
        &mut u32_to_u8_array_with_length_3(initial_handshake_packet.payload_length)?.to_vec(),
    );
    res.push(initial_handshake_packet.packet_number);
    res.push(initial_handshake_packet.protocol_version);
    let mut server_version_vec = initial_handshake_packet
        .server_version
        .into_bytes()
        .to_vec();
    server_version_vec.push(0x00);
    res.append(&mut server_version_vec);
    res.append(&mut u32_to_u8_array(initial_handshake_packet.connection_id).to_vec());
    let (mut auth_plugin_data_part1, mut auth_plugin_data_part2) =
        serialize_auth_plugin_data(initial_handshake_packet.auth_plugin_data.clone())?;
    res.append(&mut auth_plugin_data_part1);
    let mut capability_flags_buffer = u32_to_u8_array(serialize_capability_flags(
        &initial_handshake_packet.capability_flags,
    ));
    res.append(&mut capability_flags_buffer[0..2].to_vec());
    if let Some(character_set) = initial_handshake_packet.character_set {
        res.push(get_character_set_value(character_set));
    }
    if let Some(status_flags) = initial_handshake_packet.status_flags {
        res.append(&mut u16_to_u8_array(serialize_status_flags(&status_flags)).to_vec());
    }
    res.append(&mut capability_flags_buffer[2..4].to_vec());
    if initial_handshake_packet.capability_flags.client_plugin_auth {
        res.push((&initial_handshake_packet.auth_plugin_data.as_bytes().len() + 1) as u8);
    } else {
        res.push(0x00);
    }
    res.append(&mut [0x00; 10].to_vec());
    if initial_handshake_packet
        .capability_flags
        .client_secure_connection
    {
        res.append(&mut auth_plugin_data_part2);
    }
    if initial_handshake_packet.capability_flags.client_plugin_auth {
        let mut auth_plugin_name_vec = initial_handshake_packet.auth_plugin_name.into_bytes();
        auth_plugin_name_vec.push(0x00);
        res.append(&mut auth_plugin_name_vec);
    }

    if res.len() - MYSQL_PACKET_HEADER_LENGTH != initial_handshake_packet.payload_length as usize {
        return Err(ImmuxError::MySQLSerializer(
            MySQLSerializeError::SerializeInitialHandshakePacketError,
        ));
    }

    Ok(res)
}

#[cfg(test)]
mod initial_handshake_packet_tests {

    use crate::cortices::mysql::capability_flags::CapabilityFlags;
    use crate::cortices::mysql::initial_handshake_packet::{
        serialize_auth_plugin_data, serialize_initial_handshake_packet, CharacterSet,
        InitialHandshakePacket, ServerStatusFlags,
    };
    use std::str;

    #[test]
    fn test_serialize_auth_plugin_data() {
        let auth_plugin_data = r#"3h:Jo4\tyJy\177=Y)#rfIa%"#.to_string();
        let (part1, part2) = serialize_auth_plugin_data(auth_plugin_data).unwrap();
        assert_eq!(part1.len(), 9);
        assert_eq!(part2.len(), 13);
        assert_eq!(str::from_utf8(&part1[0..8]).unwrap(), r#"3h:Jo4\t"#);
        assert_eq!(str::from_utf8(&part2[0..12]).unwrap(), r#"yJy\177=Y)#r"#);
    }

    #[test]
    fn test_serialize_initial_handshake_packet() {
        let payload_length = 74;
        let packet_number = 0;
        let protocol_version = 10;
        let server_version = "8.0.15".to_string();
        let connection_id = 8;
        let auth_plugin_data = r#"3h:Jo4\tyJy\177=Y)#rfIa%"#.to_string();
        let capability_flags = CapabilityFlags {
            client_long_password: true,
            client_found_rows: true,
            client_long_flag: true,
            client_connect_with_db: true,
            client_no_schema: true,
            client_compress: true,
            client_odbc: true,
            client_local_files: true,
            client_ignore_space: true,
            client_protocol_41: true,
            client_interactive: true,
            client_ssl: true,
            client_ignore_sigpipe: true,
            client_transactions: true,
            client_reserved: true,
            client_secure_connection: true,
            client_multi_statements: true,
            client_multi_results: true,
            client_ps_multi_results: true,
            client_plugin_auth: true,
            client_connect_attrs: true,
            client_plugin_auth_lenenc_client_data: true,
            client_can_handle_expired_passwords: true,
            client_session_track: true,
            client_deprecate_eof: true,
        };
        let character_set = Some(CharacterSet::Utf8GeneralCi);
        let status_flags = Some(ServerStatusFlags {
            intrans: false,
            autocommit: true,
            more_results_exists: false,
            no_good_index_used: false,
            no_index_used: false,
            cursor_exists: false,
            last_row_sent: false,
            db_dropped: false,
            no_backslash_escapes: false,
            metadata_changed: false,
            query_was_slow: false,
            ps_out_params: false,
            intrans_readonly: false,
            session_state_changed: false,
        });
        let auth_plugin_name = "caching_sha2_password".to_string();

        let initial_handshake_packet = InitialHandshakePacket {
            payload_length,
            packet_number,
            protocol_version,
            server_version,
            connection_id,
            auth_plugin_data,
            capability_flags,
            character_set,
            status_flags,
            auth_plugin_name,
        };

        let res = serialize_initial_handshake_packet(initial_handshake_packet).unwrap();

        //        packet's length;
        assert_eq!(res[0], 0x4a);
        //        protocol version.
        assert_eq!(res[4], 0x0a);
        //        connection id.
        assert_eq!(res[12], 0x08);
        //        end of auth data part1.
        assert_eq!(res[24], 0x00);
        //        capability flags.
        assert_eq!(res[25], 0xff);
        assert_eq!(res[26], 0xff);
        //        character set.
        assert_eq!(res[27], 0x21);
        //        server status.
        assert_eq!(res[28], 0x02);
        assert_eq!(res[29], 0x00);
        //        reserved bytes.
        assert_eq!(&res[33..43], [0x00; 10])
    }
}
