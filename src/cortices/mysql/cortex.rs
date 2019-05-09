use std::net::TcpStream;

use crate::config::UnumDBConfiguration;
use crate::cortices::mysql::auth_switch_request::{
    serialize_auth_switch_request, AuthSwitchRequest,
};
use crate::cortices::mysql::auth_switch_response::parse_auth_switch_response;
use crate::cortices::mysql::capability_flags::CapabilityFlags;
use crate::cortices::mysql::character_set::CharacterSet;
use crate::cortices::mysql::error::MySQLParserError;
use crate::cortices::mysql::handshake_response_41::{
    parse_handshake_response, save_handshake_response,
};
use crate::cortices::mysql::initial_handshake_packet::{
    serialize_initial_handshake_packet, InitialHandshakePacket,
};
use crate::cortices::mysql::ok_packet::{serialize_ok_packet, HeaderOption, OkPacket};
use crate::cortices::mysql::server_status_flags::{
    save_server_status_flags, serialize_status_flags, ServerStatusFlags,
};
use crate::cortices::mysql::utils::{get_packet_number, ConnectionStatePhase};
use crate::cortices::{Cortex, CortexResponse};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::storage::core::{CoreStore, UnumCore};
use crate::utils::{pretty_dump, u16_to_u8_array};

const MYSQL_CLIENT_CONFIG_KEY: &str = "_MYSQL_CLIENT_CONFIG";

pub fn mysql_cortex_process_incoming_message(
    bytes: &[u8],
    core: &mut UnumCore,
    _stream: &TcpStream,
    _config: &UnumDBConfiguration,
) -> UnumResult<CortexResponse> {
    pretty_dump(bytes);

    match get_packet_number(&bytes)? {
        ConnectionStatePhase::LoginRequest => {
            println!("send auth switch request.");

            save_handshake_response(&bytes, core)?;

            let payload_length = 44;
            let packet_number = 2;
            let status = 0xfe;
            let plugin_name = "mysql_native_password".to_string();
            let plugin_data = "32224d563f1b0e6f783b367f0b366f4b1b5b485700".to_string();

            let auth_switch_request = AuthSwitchRequest {
                payload_length,
                packet_number,
                status,
                plugin_name,
                plugin_data,
            };

            let res = serialize_auth_switch_request(auth_switch_request).unwrap();
            return Ok(CortexResponse::Send(res));
        }
        ConnectionStatePhase::AuthSwitchResponse => {
            let auth_switch_response = parse_auth_switch_response(&bytes)?;

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

            let ok_packet_vec = serialize_ok_packet(ok_packet, core, true)?;
            let res = return Ok(CortexResponse::Send(ok_packet_vec));
        }
        _ => unimplemented!(),
    }
}

pub fn mysql_cortex_process_first_connection(core: &mut UnumCore) -> UnumResult<CortexResponse> {
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

    if let Some(data) = initial_handshake_packet.status_flags {
        let server_status_flags = serialize_status_flags(&data);
        let server_status_flags_buffer = u16_to_u8_array(server_status_flags);
        let server_status_flags_buffer =
            save_server_status_flags(&server_status_flags_buffer, core);
    } else {
        return Err(UnumError::MySQLParser(
            MySQLParserError::CannotSetServerStatusFlags,
        ));
    }

    let res = serialize_initial_handshake_packet(initial_handshake_packet).unwrap();
    return Ok(CortexResponse::Send(res));
}

pub const MYSQL_CORTEX: Cortex = Cortex {
    process_incoming_message: mysql_cortex_process_incoming_message,
    process_first_connection: Some(mysql_cortex_process_first_connection),
};
