use crate::cortices::mysql::capability_flags::{serialize_capability_flags, CapabilityFlags};
use crate::cortices::mysql::character_set::get_character_set_value;
use crate::cortices::mysql::character_set::CharacterSet;
use crate::cortices::mysql::error::MySQLSerializeError::SerializeAuthPluginDataError;
use crate::cortices::mysql::utils::u32_to_u8_array_with_length_3;
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::{set_bit_u16, u16_to_u8_array, u32_to_u8_array};

/// @see https://dev.mysql.com/doc/internals/en/status-flags.html#packet-Protocol::StatusFlags
pub struct ServerStatusFlags {
    pub intrans: bool,
    pub autocommit: bool,
    pub more_results_exists: bool,
    pub no_good_index_used: bool,
    pub no_index_used: bool,
    pub cursor_exists: bool,
    pub last_row_sent: bool,
    pub db_dropped: bool,
    pub no_backslash_escapes: bool,
    pub metadata_changed: bool,
    pub query_was_slow: bool,
    pub ps_out_params: bool,
    pub intrans_readonly: bool,
    pub session_state_changed: bool,
}

pub struct InitialHandshakePacket {
    pub packet_length: u32,
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

pub fn serialize_status_flags(flags_struct: &ServerStatusFlags) -> u16 {
    let mut result: u16 = 0;
    set_bit_u16(&mut result, 0, flags_struct.intrans);
    set_bit_u16(&mut result, 1, flags_struct.autocommit);
    set_bit_u16(&mut result, 2, flags_struct.more_results_exists);
    set_bit_u16(&mut result, 3, flags_struct.no_good_index_used);
    set_bit_u16(&mut result, 4, flags_struct.no_index_used);
    set_bit_u16(&mut result, 5, flags_struct.cursor_exists);
    set_bit_u16(&mut result, 6, flags_struct.last_row_sent);
    set_bit_u16(&mut result, 7, flags_struct.db_dropped);
    set_bit_u16(&mut result, 8, flags_struct.no_backslash_escapes);
    set_bit_u16(&mut result, 9, flags_struct.metadata_changed);
    set_bit_u16(&mut result, 10, flags_struct.query_was_slow);
    set_bit_u16(&mut result, 11, flags_struct.ps_out_params);
    set_bit_u16(&mut result, 12, flags_struct.intrans_readonly);
    set_bit_u16(&mut result, 13, flags_struct.session_state_changed);

    return result;
}

pub fn serialize_auth_plugin_data(auth_plugin_data: String) -> UnumResult<(Vec<u8>, Vec<u8>)> {
    let auth_plugin_data_vec = auth_plugin_data.into_bytes();
    if auth_plugin_data_vec.is_empty() {
        return Err(UnumError::MySQLSerializer(SerializeAuthPluginDataError));
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
) -> UnumResult<Vec<u8>> {
    let mut res = Vec::new();
    res.append(
        &mut u32_to_u8_array_with_length_3(initial_handshake_packet.packet_length)?.to_vec(),
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

    Ok(res)
}

#[cfg(test)]
mod initial_handshake_packet_tests {

    use crate::cortices::mysql::capability_flags::CapabilityFlags;
    use crate::cortices::mysql::initial_handshake_packet::{
        serialize_auth_plugin_data, serialize_initial_handshake_packet, serialize_status_flags,
        CharacterSet, InitialHandshakePacket, ServerStatusFlags,
    };
    use std::str;

    #[test]
    fn test_serialize_status_flags() {
        let status_flags = ServerStatusFlags {
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
        };
        assert_eq!(serialize_status_flags(&status_flags), 0x0002);
    }

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
        let packet_length = 78;
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
            packet_length,
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
        assert_eq!(res[0], 0x4e);
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
