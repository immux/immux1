use crate::cortices::utils::parse_u32;
use crate::declarations::errors::UnumResult;
use crate::utils::{get_bit_u32, set_bit_u32};

/// @see https://dev.mysql.com/doc/internals/en/capability-flags.html#packet-Protocol::CapabilityFlags
pub struct CapabilityFlags {
    pub client_long_password: bool,
    pub client_found_rows: bool,
    pub client_long_flag: bool,
    pub client_connect_with_db: bool,
    pub client_no_schema: bool,
    pub client_compress: bool,
    pub client_odbc: bool,
    pub client_local_files: bool,
    pub client_ignore_space: bool,
    pub client_protocol_41: bool,
    pub client_interactive: bool,
    pub client_ssl: bool,
    pub client_ignore_sigpipe: bool,
    pub client_transactions: bool,
    pub client_reserved: bool,
    pub client_secure_connection: bool,
    pub client_multi_statements: bool,
    pub client_multi_results: bool,
    pub client_ps_multi_results: bool,
    pub client_plugin_auth: bool,
    pub client_connect_attrs: bool,
    pub client_plugin_auth_lenenc_client_data: bool,
    pub client_can_handle_expired_passwords: bool,
    pub client_session_track: bool,
    pub client_deprecate_eof: bool,
}

pub fn parse_capability_flags(buffer: &[u8]) -> UnumResult<(CapabilityFlags, usize)> {
    let (capability_flags_vec, index_offset) = parse_u32(&buffer)?;
    let client_long_password = get_bit_u32(capability_flags_vec, 0);
    let client_found_rows = get_bit_u32(capability_flags_vec, 1);
    let client_long_flag = get_bit_u32(capability_flags_vec, 2);
    let client_connect_with_db = get_bit_u32(capability_flags_vec, 3);
    let client_no_schema = get_bit_u32(capability_flags_vec, 4);
    let client_compress = get_bit_u32(capability_flags_vec, 5);
    let client_odbc = get_bit_u32(capability_flags_vec, 6);
    let client_local_files = get_bit_u32(capability_flags_vec, 7);
    let client_ignore_space = get_bit_u32(capability_flags_vec, 8);
    let client_protocol_41 = get_bit_u32(capability_flags_vec, 9);
    let client_interactive = get_bit_u32(capability_flags_vec, 10);
    let client_ssl = get_bit_u32(capability_flags_vec, 11);
    let client_ignore_sigpipe = get_bit_u32(capability_flags_vec, 12);
    let client_transactions = get_bit_u32(capability_flags_vec, 13);
    let client_reserved = get_bit_u32(capability_flags_vec, 14);
    let client_secure_connection = get_bit_u32(capability_flags_vec, 15);
    let client_multi_statements = get_bit_u32(capability_flags_vec, 16);
    let client_multi_results = get_bit_u32(capability_flags_vec, 17);
    let client_ps_multi_results = get_bit_u32(capability_flags_vec, 18);
    let client_plugin_auth = get_bit_u32(capability_flags_vec, 19);
    let client_connect_attrs = get_bit_u32(capability_flags_vec, 20);
    let client_plugin_auth_lenenc_client_data = get_bit_u32(capability_flags_vec, 21);
    let client_can_handle_expired_passwords = get_bit_u32(capability_flags_vec, 22);
    let client_session_track = get_bit_u32(capability_flags_vec, 23);
    let client_deprecate_eof = get_bit_u32(capability_flags_vec, 24);

    let capability_flags = CapabilityFlags {
        client_long_password,
        client_found_rows,
        client_long_flag,
        client_connect_with_db,
        client_no_schema,
        client_compress,
        client_odbc,
        client_local_files,
        client_ignore_space,
        client_protocol_41,
        client_interactive,
        client_ssl,
        client_ignore_sigpipe,
        client_transactions,
        client_reserved,
        client_secure_connection,
        client_multi_statements,
        client_multi_results,
        client_ps_multi_results,
        client_plugin_auth,
        client_connect_attrs,
        client_plugin_auth_lenenc_client_data,
        client_can_handle_expired_passwords,
        client_session_track,
        client_deprecate_eof,
    };
    Ok((capability_flags, index_offset))
}

pub fn serialize_capability_flags(flags_struct: &CapabilityFlags) -> u32 {
    let mut result: u32 = 0;
    set_bit_u32(&mut result, 0, flags_struct.client_long_password);
    set_bit_u32(&mut result, 1, flags_struct.client_found_rows);
    set_bit_u32(&mut result, 2, flags_struct.client_long_flag);
    set_bit_u32(&mut result, 3, flags_struct.client_connect_with_db);
    set_bit_u32(&mut result, 4, flags_struct.client_no_schema);
    set_bit_u32(&mut result, 5, flags_struct.client_compress);
    set_bit_u32(&mut result, 6, flags_struct.client_odbc);
    set_bit_u32(&mut result, 7, flags_struct.client_local_files);
    set_bit_u32(&mut result, 8, flags_struct.client_ignore_space);
    set_bit_u32(&mut result, 9, flags_struct.client_protocol_41);
    set_bit_u32(&mut result, 10, flags_struct.client_interactive);
    set_bit_u32(&mut result, 11, flags_struct.client_ssl);
    set_bit_u32(&mut result, 12, flags_struct.client_ignore_sigpipe);
    set_bit_u32(&mut result, 13, flags_struct.client_transactions);
    set_bit_u32(&mut result, 14, flags_struct.client_reserved);
    set_bit_u32(&mut result, 15, flags_struct.client_secure_connection);
    set_bit_u32(&mut result, 16, flags_struct.client_multi_statements);
    set_bit_u32(&mut result, 17, flags_struct.client_multi_results);
    set_bit_u32(&mut result, 18, flags_struct.client_ps_multi_results);
    set_bit_u32(&mut result, 19, flags_struct.client_plugin_auth);
    set_bit_u32(&mut result, 20, flags_struct.client_connect_attrs);
    set_bit_u32(
        &mut result,
        21,
        flags_struct.client_plugin_auth_lenenc_client_data,
    );
    set_bit_u32(
        &mut result,
        22,
        flags_struct.client_can_handle_expired_passwords,
    );
    set_bit_u32(&mut result, 23, flags_struct.client_session_track);
    set_bit_u32(&mut result, 24, flags_struct.client_deprecate_eof);

    return result;
}

#[cfg(test)]
mod capability_flags_tests {
    use crate::cortices::mysql::capability_flags::{
        parse_capability_flags, serialize_capability_flags, CapabilityFlags,
    };

    #[test]
    fn test_serialize_capability_flags() {
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
        assert_eq!(serialize_capability_flags(&capability_flags), 33554431);
    }

    #[test]
    fn test_parse_capability_flags() {
        let capability_flags_buffer = [0x85, 0xa6, 0xff, 0x01];
        let (capability_flags, index_offset) =
            parse_capability_flags(&capability_flags_buffer).unwrap();
        assert_eq!(index_offset, 4);
        assert_eq!(capability_flags.client_long_password, true);
        assert_eq!(capability_flags.client_found_rows, false);
        assert_eq!(capability_flags.client_long_flag, true);
        assert_eq!(capability_flags.client_connect_with_db, false);
        assert_eq!(capability_flags.client_no_schema, false);
        assert_eq!(capability_flags.client_compress, false);
        assert_eq!(capability_flags.client_odbc, false);
        assert_eq!(capability_flags.client_local_files, true);
        assert_eq!(capability_flags.client_ignore_space, false);
        assert_eq!(capability_flags.client_protocol_41, true);
        assert_eq!(capability_flags.client_interactive, true);
        assert_eq!(capability_flags.client_ssl, false);
        assert_eq!(capability_flags.client_ignore_sigpipe, false);
        assert_eq!(capability_flags.client_transactions, true);
        assert_eq!(capability_flags.client_reserved, false);
        assert_eq!(capability_flags.client_secure_connection, true);
        assert_eq!(capability_flags.client_multi_statements, true);
        assert_eq!(capability_flags.client_multi_results, true);
        assert_eq!(capability_flags.client_ps_multi_results, true);
        assert_eq!(capability_flags.client_plugin_auth, true);
        assert_eq!(capability_flags.client_connect_attrs, true);
        assert_eq!(capability_flags.client_plugin_auth_lenenc_client_data, true);
        assert_eq!(capability_flags.client_can_handle_expired_passwords, true);
        assert_eq!(capability_flags.client_session_track, true);
        assert_eq!(capability_flags.client_deprecate_eof, true);
    }
}
